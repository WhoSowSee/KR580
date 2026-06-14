use crate::decode_oem_text;
use printpdf::{
    Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, TextItem,
};
use std::path::Path;

const FONT_BYTES: &[u8] = include_bytes!("../../../../assets/fonts/RobotoMono.ttf");
const PAGE_WIDTH: Mm = Mm(210.0);
const PAGE_HEIGHT: Mm = Mm(297.0);
const MARGIN: Mm = Mm(18.0);
const FONT_SIZE: Pt = Pt(10.0);
const LINE_HEIGHT: Pt = Pt(13.0);
const COLUMNS_PER_LINE: usize = 80;
const LINES_PER_PAGE: usize = 56;

pub(super) fn write(path: &Path, spool: &[u8]) -> Result<(), String> {
    let font = ParsedFont::from_bytes(FONT_BYTES, 0, &mut Vec::new())
        .ok_or_else(|| "failed to load printer font".to_owned())?;
    let mut document = PdfDocument::new("KR580 printer output");
    let font_id = document.add_font(&font);
    let lines = printer_lines(spool);
    let pages = lines
        .chunks(LINES_PER_PAGE)
        .map(|page_lines| {
            let mut operations = vec![
                Op::StartTextSection,
                Op::SetFont {
                    font: PdfFontHandle::External(font_id.clone()),
                    size: FONT_SIZE,
                },
                Op::SetLineHeight { lh: LINE_HEIGHT },
                Op::SetTextCursor {
                    pos: Point {
                        x: MARGIN.into(),
                        y: Mm(PAGE_HEIGHT.0 - MARGIN.0).into(),
                    },
                },
            ];
            for (index, line) in page_lines.iter().enumerate() {
                if index != 0 {
                    operations.push(Op::AddLineBreak);
                }
                operations.push(Op::ShowText {
                    items: vec![TextItem::Text(line.clone())],
                });
            }
            operations.push(Op::EndTextSection);
            PdfPage::new(PAGE_WIDTH, PAGE_HEIGHT, operations)
        })
        .collect();
    let bytes = document
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());
    std::fs::write(path, bytes).map_err(|error| error.to_string())
}

fn printer_lines(spool: &[u8]) -> Vec<String> {
    let text = decode_oem_text(spool).replace('\t', "    ");
    let mut lines = Vec::new();
    for logical_line in text.split('\n') {
        let characters = logical_line.chars().collect::<Vec<_>>();
        if characters.is_empty() {
            lines.push(String::new());
            continue;
        }
        lines.extend(
            characters
                .chunks(COLUMNS_PER_LINE)
                .map(|chunk| chunk.iter().collect()),
        );
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::printer_lines;

    #[test]
    fn printer_lines_preserve_blank_lines_and_wrap_at_eighty_columns() {
        let source = format!("{}\r\n\r\nB", "A".repeat(81));
        let lines = printer_lines(source.as_bytes());

        assert_eq!(lines[0], "A".repeat(80));
        assert_eq!(lines[1], "A");
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], "B");
    }
}
