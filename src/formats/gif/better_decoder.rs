// Decodes a *valid* gif byte stream into one or more images.

use std::{error::Error, io::Read};

use crate::{
    colors::rgb::RGB,
    formats::gif::blocks::GraphicControlExtension,
    generic_image::{GenericImage, GenericImageMut},
    image_buffer::ImageBuffer,
};

use super::{
    blocks::{
        BlockLabel, BlockLabelType, BlockSeparator, ColorTable, ColorTableLookup, ControlBlocks,
        GraphicRenderingBlocks, Header, LogicalScreenDescriptor, SpecialPurposeBlocks,
        TableBasedImage,
    },
    errors::GIFParseError,
    gif::{DisposalMethod, GIFDecode, Version},
    ringbuf::RingBuffer,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ReadNext {
    /// HAVE: nothing.
    ///
    /// READ: Header.
    ///
    /// GOTO: LogicalScreenDescriptor.
    Header,
    /// HAVE Read header.
    ///
    /// READ: LogicalScreenDescriptor.
    ///
    /// GOTO: if global_color_table_flag is set then GlobalColorTable else
    /// |   BlockType(NoRestriction).
    LogicalScreenDescriptor,
    /// HAVE: LogicalScreenDescriptor with global_color_table_flag enabled.
    ///
    /// READ: GlobalColorTable.
    ///
    /// GOTO: BlockType(NoRestriction)
    GlobalColorTable(u8, bool),
    /// HAVE: LogicalScreenDescriptor + (GlobalColorTable) + (BlockRestriction).
    ///
    /// |    BlockRestrictions could occur as we have read some block (e.g GraphicsControlExtension)
    /// |    which after the rules of the GIF Grammar 89a must be followed by a GraphicsBlock.
    ///
    /// READ: BlockType (1st byte of the block either Image Seperator, Extension Introducer or Trailor).
    ///
    /// GOTO:
    /// |   if ImageSeperator and BlockRestriction is NoRestriction or GraphicsBlock then GraphicsBlock(TableBasedImage)
    /// |   if ExtensionIntroducer then ExtensionType(BlockRestriction)
    /// |   if Trailor and Restriction is NoRestriction then End
    /// |   else RestrictionError
    BlockType(Option<BlockLabelType>),

    /// HAVE: BlockType of Extension + (BlockRestriction).
    ///
    /// |    BlockRestrictions could occur as we have read some block (e.g GraphicsControlExtension)
    /// |    which after the rules of the GIF Grammar 89a must be followed by a GraphicsBlock and
    /// |    PlainTextExtension is a GraphicsBlock
    ///
    /// READ: ExtensionLabel. Follows the Label Byte Ordering of GIF Spec: 0x00-0x7F
    /// GraphicsBlocks, 0x80-0xF9 Control Blocks, 0xFA-0xFF Special Purpose Blocks
    ///
    /// GOTO:
    /// If the BlockRestriction is set only accept labels of that block type else give out an RestrictionError and abort
    /// |   if PlainTextExtension then GraphicBlock(PlainTextExtension).
    /// |   if any other graphics rendering block label then GraphicsBlock(UnknownExtension)
    /// |   if GraphicsControlExtension then ControlBlock(GraphicsControlExtension)
    /// |   if any other control block label then ControlBlock(UnknownExtension)
    /// |   if ApplicationExtension then SpecialPurposeBlock(ApplicationExtension)
    /// |   if CommentExtension then SpecialPurposeBlock(CommentExtension)
    /// |   if any other special purpose block then SpecialPurposeBlock(UnknownExtension)
    /// |   else InvalidExtensionBlockLabel
    ExtensionType(Option<BlockLabelType>),

    /// HAVE: some Control Extension (we dont treat ImageDescriptor or the LocalColorTable as
    /// Control Blocks due to the necassity of the in a Table Based Image).
    ///
    /// READ: The control block and add it to the decoder state or skip it if unkown
    ///
    /// GOTO:
    /// |   if GraphicsControlExtension then BlockType(GraphicRenderingBlockRestriction)
    /// |   else BlockType(NoRestriction)
    ControlBlock(ControlBlocks),

    /// HAVE: some Graphic Block. (We treat the entire TableBasedImage as a GraphicBlock as
    /// these have to come after one another).
    ///
    /// READ: the image / text and add them to the finished decoded image(s) of the decoder.
    ///
    /// GOTO: BlockType(NoRestriction)
    GraphicsBlock(GraphicRenderingBlocks),
    /// HAVE: some SpecialPurpose Extension.
    ///
    /// READ: read it to the optional decoder state or skip it if unknown
    ///
    /// GOTO: BlockType(NoRestriction)
    SpecialPurposeBlock(SpecialPurposeBlocks),

    /// HAVE: Trailor
    ///
    /// READ: Nothing
    ///
    /// GOTO: End(Hopefully not we should finish here)
    End,
}

enum ReadNextSubblock {
    SubblockType,
    //Read GraphicRenderingBlock (either TableBasedImage or PlainTextExtension)
    GraphicRendering,

    TableBasedImage,
    ImageDescriptor,
    LocalColorTable,
    ImageData,
}

#[derive(Debug, Copy, Clone)]
struct GIFDecoderState {
    /// The global color table to use. If `None` the `default_color_table` must be used
    global_color_table: Option<ColorTable>,
    version: Version,
    logical_dim: (u16, u16),
    color_resolution: u8,
    pixel_aspect_ration: u8,
    background_color_ration: u8,

    /// The point in which we are in the grammar parsing of the stream
    grammar_state: ReadNext,
    /// The last and the current GraphicControlExtension as a "ring buffer"
    graphic_controls: RingBuffer<GraphicControlExtension, 2>,
}

//TODO: Enable switching the decompression.

struct GIFDecoder<'a, R: Read + Copy> {
    /// The state of the current decoding
    state: GIFDecoderState,
    /// A reference to the default color table to use. !Should not be changed after initialization
    default_color_table: Option<&'a ColorTable>,
    current_image: Option<ImageBuffer<RGB<u8>, Vec<u8>>>,
    is_animation: bool,
    reader: R,
}

impl<R: Read + Copy> GIFDecoder<'_, R> {
    fn new(reader: R) -> Self {
        GIFDecoder {
            state: GIFDecoderState {
                global_color_table: None,
                version: Version::Version89a,
                logical_dim: (0, 0),
                color_resolution: 0,
                pixel_aspect_ration: 0,
                background_color_ration: 0,
                grammar_state: ReadNext::Header,
                graphic_controls: RingBuffer::new(),
            },
            default_color_table: None,
            current_image: None,
            is_animation: false,
            reader,
        }
    }

    fn next_state(&mut self) -> Result<ReadNext, GIFParseError> {
        match self.state.grammar_state {
            ReadNext::Header => {
                let header = Header::parse(&mut self.reader)?;
                self.state.version = header.version;

                println!("HEADER: {:#?}", header);

                Ok(ReadNext::LogicalScreenDescriptor)
            }
            ReadNext::LogicalScreenDescriptor => {
                // This is infallible. It could be wrong size or so but we won't now (yet).
                let descriptor = LogicalScreenDescriptor::parse(&mut self.reader)?;
                self.state.pixel_aspect_ration = descriptor.pixel_aspect_ratio;
                self.state.logical_dim = (
                    descriptor.logical_screen_width,
                    descriptor.logical_screen_height,
                );
                self.state.color_resolution = descriptor.color_resolution();

                println!("LogicalScreenDescriptor: {:#?}", descriptor);

                if descriptor.global_color_table_flag() {
                    Ok(ReadNext::GlobalColorTable(
                        descriptor.global_color_table_size(),
                        descriptor.sort_flag(),
                    ))
                } else {
                    Ok(ReadNext::BlockType(None))
                }
            }
            ReadNext::GlobalColorTable(size, sorted) => {
                let table = ColorTable::try_from_reader(&mut self.reader, size, sorted)?;
                self.state.global_color_table = Some(table);
                println!("Global Color Table : {}", table);
                Ok(ReadNext::BlockType(None))
            }
            ReadNext::BlockType(restriction) => {
                let mut buf = [0u8; 1];

                self.reader
                    .read_exact(&mut buf)
                    .map_err(|err| GIFParseError::Io {
                        reason: "io error (likely EOF) during block type label reading".to_string(),
                        cause: err,
                    })?;

                let separator = BlockSeparator::try_from_u8(buf[0])
                    .ok_or(GIFParseError::UnexpectedBlockDiscriminant(buf[0]))?;

                if restriction
                    .is_some_and(|restriction_type| separator.can_be_type(restriction_type))
                {
                    return Err(GIFParseError::UnexpectedBlockDiscriminant(buf[0]));
                }

                match separator {
                    BlockSeparator::Image => Ok(ReadNext::GraphicsBlock(
                        GraphicRenderingBlocks::TableBasedImage,
                    )),
                    BlockSeparator::Extension => Ok(ReadNext::ExtensionType(restriction)),
                    BlockSeparator::Trailer => Ok(ReadNext::End),
                }
            }
            ReadNext::ExtensionType(restriction) => {
                let mut buf = [0u8; 1];
                self.reader
                    .read_exact(&mut buf)
                    .map_err(|err| GIFParseError::Io {
                        reason: "io error during extension label read".to_string(),
                        cause: err,
                    })?;

                let label_type = BlockLabelType::from(buf[0]);

                if restriction.is_some_and(|restriction_type| label_type != restriction_type) {
                    return Err(GIFParseError::UnexpectedExtensionLabel(buf[0]));
                }

                let label = BlockLabel::try_from_u8(buf[0]);
                match label_type {
                    BlockLabelType::Graphic => {
                        let graphic_label = match label {
                            Some(block) => GraphicRenderingBlocks::try_from(block)
                                .expect("expected block to be of type graphic"),
                            None => GraphicRenderingBlocks::UnknownBlock,
                        };

                        Ok(ReadNext::GraphicsBlock(graphic_label))
                    }
                    BlockLabelType::Control => {
                        let control_label = match label {
                            Some(block) => ControlBlocks::try_from(block)
                                .expect("expected block to be of type graphic"),
                            None => ControlBlocks::UnknownBlock,
                        };

                        Ok(ReadNext::ControlBlock(control_label))
                    }
                    BlockLabelType::SpecialPurpose => {
                        let special_purpose_label = match label {
                            Some(block) => SpecialPurposeBlocks::try_from(block)
                                .expect("expected block to be of type special purpose"),
                            None => SpecialPurposeBlocks::UnknownBlock,
                        };

                        Ok(ReadNext::SpecialPurposeBlock(special_purpose_label))
                    }
                    _ => Err(GIFParseError::UnexpectedExtensionLabel(buf[0])),
                }
            }
            ReadNext::ControlBlock(block_type) => {
                match block_type {
                    ControlBlocks::GraphicsControlExtension => {
                        self.process_graphic_control_extension()?
                    }
                    ControlBlocks::UnknownBlock => panic!("unknown control block"),
                }
                Ok(ReadNext::BlockType(None))
            }
            ReadNext::GraphicsBlock(block_type) => {
                match block_type {
                    GraphicRenderingBlocks::TableBasedImage => self.process_table_based_image()?,
                    GraphicRenderingBlocks::PlainTextExtension => {
                        self.process_plain_text_extension()
                    }
                    GraphicRenderingBlocks::UnknownBlock => panic!("unknown graphic block"),
                }
                Ok(ReadNext::BlockType(None))
            }
            ReadNext::SpecialPurposeBlock(_block_type) => {
                match _block_type {
                    SpecialPurposeBlocks::CommentExtension => self.process_comment_extension(),
                    SpecialPurposeBlocks::ApplicationExtension => {
                        self.process_application_extension()
                    }
                    SpecialPurposeBlocks::UnknownBlock => {
                        panic!("unknown special purpose block extension")
                    }
                }
                Ok(ReadNext::BlockType(None))
            }
            ReadNext::End => Ok(ReadNext::End),
        }
    }

    /// Processes a table based image i.e. an image descriptor (with Local Color Table) and then the
    /// (LZW Compressed) Image Data.
    ///
    /// Then parses the image with the current color table and interlace flags to the
    /// `self.current_image`.
    ///
    /// Returns an empty result, since all image data is stored in `self.current_image`.
    fn process_table_based_image(&mut self) -> Result<(), GIFParseError> {
        // What we have to do here heavily depends on the previous graphic control extension (if
        // any)

        // let active_control = self.state.active_graphic_control.unwrap_or_default();
        //
        // if active_control.delay_time() > 0 {
        //     self.is_animation = true;
        // }
        //
        // if active_control.delay_time() > 0 || self.current_image.is_none() {
        //     // Either we must have a new frame (delay_time)
        //     self.create_image(active_control.disposal_method());
        // }

        // Get the current image or create one if not already
        if self.current_image.is_none() {
            self.current_image = Some(ImageBuffer::<RGB<u8>, Vec<u8>>::new(
                self.state.logical_dim.0 as u32,
                self.state.logical_dim.1 as u32,
            ))
        }
        let image = self.current_image.as_mut().unwrap();

        let table_based_image = TableBasedImage::parse(&mut self.reader)?;

        // Get the color table Local > Global > Default
        let color_table = table_based_image
            .local_color_table_ref()
            .or(self.state.global_color_table.as_ref())
            .or(self.default_color_table)
            .unwrap();

        if table_based_image.descriptor().interlace_flag() {
            //draw interlaced
        } else {
            //draw continuous
            // OPTIMIZE: this
            let pixels = table_based_image
                .data()
                .iter()
                .map(|color_index| color_table.lookup_fallback(*color_index))
                .collect::<Vec<_>>();

            if let Err(_e) = image.put_rect(
                table_based_image.descriptor().image_position().left as u32,
                table_based_image.descriptor().image_position().top as u32,
                table_based_image.descriptor().image_dim().0 as u32,
                table_based_image.descriptor().image_dim().1 as u32,
                &pixels,
            ) {
                // pixels.len < width * height
                return Err(GIFParseError::ImageDataError);
            }
        }

        Ok(())
    }

    fn process_graphic_control_extension(&mut self) -> Result<(), GIFParseError> {
        let block = GraphicControlExtension::parse(&mut self.reader)?;

        block.disposal_method();
        todo!();
    }

    fn process_plain_text_extension(&mut self) {
        todo!();
    }

    fn process_comment_extension(&mut self) {
        todo!();
    }
    fn process_application_extension(&mut self) {
        todo!();
    }
}

impl<R: Read + Copy> GIFDecode for GIFDecoder<'_, R> {
    fn decode(mut self) -> Result<super::gif::GIFImage, GIFParseError> {
        self.state.grammar_state = ReadNext::Header;

        let seen_error = false;
        // iterate through states
        while !seen_error && self.state.grammar_state != ReadNext::End {
            //TODO: debug print currently parsing state

            println!("Parsing State [{:?}]: started", self.state.grammar_state);

            let res = self.next_state();

            if let Err(err) = res {
                //TODO: debug print
                println!("Error during parse {}", err);
                return Err(err);
            }

            match res {
                Ok(read_next) => self.state.grammar_state = read_next,
                Err(err) => {
                    //TODO: debug print error
                    return Err(err);
                }
            }

            println!("Parsing State: done")
        }

        println!("Parse GIF complete");
        Ok(super::gif::GIFImage::Single(self.current_image.unwrap()))
    }
}

#[cfg(test)]
mod test {
    use crate::formats::gif::gif::GIFDecode;

    use super::GIFDecoder;

    #[test]
    fn decode_gif() {
        let gif_data = include_bytes!("../../../test-assets/simplest.gif");

        let decoder = GIFDecoder::new(&gif_data[..]);
        let res = decoder.decode();
        if let Err(err) = res {
            panic!("Decode err {:?}", err)
        }

        println!("Decode successful");
        match res.unwrap() {
            crate::formats::gif::gif::GIFImage::None => todo!(),
            crate::formats::gif::gif::GIFImage::Single(image_buffer) => {
                println!("DEBUG IMG: {:?}", image_buffer);
            }
            crate::formats::gif::gif::GIFImage::Animation(multi_gif) => todo!(),
        }
    }
}
