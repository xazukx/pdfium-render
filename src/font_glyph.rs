//! Defines the [PdfFontGlyph] struct, exposing functionality related to a single
//! font glyph in a `PdfFontGlyphs` collection.

use crate::bindgen::{FPDF_FONT, FPDF_GLYPHPATH};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::font_glyphs::PdfFontGlyphIndex;
use crate::page::PdfPoints;
use crate::path_segment::PdfPathSegment;
use crate::path_segments::{PdfPathSegmentIndex, PdfPathSegments, PdfPathSegmentsIterator};
use std::convert::TryInto;
use std::os::raw::{c_float, c_int, c_uint};

pub struct PdfFontGlyph<'a> {
    handle: FPDF_FONT,
    index: PdfFontGlyphIndex,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfFontGlyph<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        handle: FPDF_FONT,
        index: PdfFontGlyphIndex,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        Self {
            handle,
            index,
            bindings,
        }
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfFontGlyph].
    #[inline]
    pub fn bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }

    /// Returns the width of this [PdfFontGlyph] when rendered at the given font size.
    pub fn width_at_font_size(&self, size: PdfPoints) -> PdfPoints {
        let mut width = 0.0;

        if self.bindings.is_true(self.bindings.FPDFFont_GetGlyphWidth(
            self.handle,
            self.index as c_uint,
            size.value as c_float,
            &mut width,
        )) {
            PdfPoints::new(width)
        } else {
            PdfPoints::ZERO
        }
    }

    /// Returns the path segments of this [PdfFontGlyph] when rendered at the given font size.
    pub fn segments_at_font_size(&self, size: PdfPoints) -> Result<PdfFontGlyphPath, PdfiumError> {
        let handle = self.bindings().FPDFFont_GetGlyphPath(
            self.handle,
            self.index as c_uint,
            size.value as c_float,
        );

        if handle.is_null() {
            if let Some(error) = self.bindings().get_pdfium_last_error() {
                Err(PdfiumError::PdfiumLibraryInternalError(error))
            } else {
                // This would be an unusual situation; a null handle indicating failure,
                // yet Pdfium's error code indicates success.

                Err(PdfiumError::PdfiumLibraryInternalError(
                    PdfiumInternalError::Unknown,
                ))
            }
        } else {
            Ok(PdfFontGlyphPath::from_pdfium(handle, self.bindings()))
        }
    }
}

/// The collection of [PdfPathSegment] objects inside a font glyph path.
pub struct PdfFontGlyphPath<'a> {
    handle: FPDF_GLYPHPATH,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfFontGlyphPath<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        handle: FPDF_GLYPHPATH,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        Self { handle, bindings }
    }
}

impl<'a> PdfPathSegments<'a> for PdfFontGlyphPath<'a> {
    #[inline]
    fn bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }

    #[inline]
    fn len(&self) -> PdfPathSegmentIndex {
        self.bindings()
            .FPDFGlyphPath_CountGlyphSegments(self.handle)
            .try_into()
            .unwrap_or(0)
    }

    fn get(&self, index: PdfPathSegmentIndex) -> Result<PdfPathSegment<'a>, PdfiumError> {
        let handle = self
            .bindings()
            .FPDFGlyphPath_GetGlyphPathSegment(self.handle, index as c_int);

        if handle.is_null() {
            if let Some(error) = self.bindings().get_pdfium_last_error() {
                Err(PdfiumError::PdfiumLibraryInternalError(error))
            } else {
                // This would be an unusual situation; a null handle indicating failure,
                // yet Pdfium's error code indicates success.

                Err(PdfiumError::PdfiumLibraryInternalError(
                    PdfiumInternalError::Unknown,
                ))
            }
        } else {
            Ok(PdfPathSegment::from_pdfium(handle, self.bindings()))
        }
    }

    #[inline]
    fn iter(&'a self) -> PdfPathSegmentsIterator<'a> {
        PdfPathSegmentsIterator::new(self)
    }
}
