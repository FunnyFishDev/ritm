use eframe::egui;
use egui::UiBuilder;

pub struct Grid {
    cell: egui::Rect,
    row_num: usize,
    col_num: usize,
}

impl Grid {
    pub fn new(ui: &mut egui::Ui, split_row_num: usize, split_col_num: usize) -> Self {
        // Get item spacing
        let item_spacing = ui.spacing().item_spacing.x;

        // Get max rect size
        let mut max_rect = ui.max_rect();

        // Adjust it so it does not include spacing
        // max_rect.set_width(max_rect.width() - 2.*item_spacing);

        // Get width for the widgets
        let width = max_rect.width() / (split_col_num as f32);

        // Get height for the widgets
        let height = max_rect.height() / (split_row_num as f32);

        // Compute the offset for subsequent rects
        let _offset = egui::vec2(width + item_spacing, 0.);

        // And set the width of rect
        max_rect.set_width(width);

        // And set the height of rect
        max_rect.set_height(height);

        // Return
        Self {
            cell: max_rect,
            row_num: split_row_num,
            col_num: split_col_num,
        }
    }

    pub fn place<R>(
        &self,
        ui: &mut egui::Ui,
        row: usize,
        col: usize,
        add_ctx: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        assert!(row >= 1);
        assert!(col >= 1);
        let height = self.cell.height();
        let width = self.cell.width();
        let colf32: f32 = (col - 1) as f32;
        let rowf32: f32 = (row - 1) as f32;
        let hold_rect = self
            .cell
            .translate(egui::vec2(colf32 * width, rowf32 * height));
        // Return
        ui.scope_builder(UiBuilder::new().max_rect(hold_rect), add_ctx)
    }
}
