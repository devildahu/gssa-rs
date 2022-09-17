//! Manage text layout on screen

use crate::textdraw::{Draw, Pos};

#[derive(Copy, Clone)]
enum Axis {
    X,
    Y,
}
struct ToChange<'a> {
    axis: Axis,
    x: &'a mut usize,
    y: &'a mut usize,
}
/// State of layouting
impl<'a> ToChange<'a> {
    #[inline]
    const fn current_axis(&mut self) -> &mut usize {
        match self.axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }
    #[inline]
    const fn add_rect(&mut self, width: usize, height: usize) {
        let value = match self.axis {
            Axis::X => width,
            Axis::Y => height,
        };
        let to_change = self.current_axis();
        *to_change += value;
    }
    #[inline]
    const fn add(&mut self, value: usize) {
        let to_change = self.current_axis();
        *to_change += value;
    }
    #[inline]
    const fn pos(&self) -> Pos {
        Pos {
            x: *self.x,
            y: *self.y,
        }
    }
    #[inline]
    fn vertical(&mut self, f: impl FnOnce(&mut ToChange)) {
        let old_axis = self.axis;
        let old_y = *self.y;
        self.axis = Axis::Y;
        f(self);
        self.axis = old_axis;
        *self.y = old_y;
    }
    #[inline]
    fn horizontal(&mut self, f: impl FnOnce(&mut ToChange)) {
        let old_axis = self.axis;
        let old_x = *self.x;
        self.axis = Axis::X;
        f(self);
        self.axis = old_axis;
        *self.x = old_x;
    }
}
macro_rules! layout {
    (@hint $to_change:ident space ($count:literal)) => {{
        $to_change.add($count);
    }};
    (@hint $to_change:ident text ($text:literal)) => {{
        $text.draw($to_change.pos());
        let text_width = $text.split('\n').map(|line| line.len()).max().unwrap_or(0);
        let text_height = $text.chars().filter(|char| *char ==  '\n').count() + 1;
        $to_change.add_rect(text_width, text_height);
    }};
    (@hint $to_change:ident select ($refer:expr, $text:literal)) => {{
        $text.draw($to_change.pos());
        *$refer = $to_change.pos();
        let text_width = $text.split('\n').map(|line| line.len()).max().unwrap_or(0);
        let text_height = $text.chars().filter(|char| *char ==  '\n').count() + 1;
        $to_change.add_rect(text_width, text_height);
    }};
    (@hint $to_change:ident vertical ($( $hint:ident $hint_args:tt ),+ $(,)?)) => {{
        $to_change.vertical(|to_change| {
            $( layout!( @hint to_change $hint $hint_args ); )*
        });
    }};
    (@hint $to_change:ident horizontal ($( $hint:ident $hint_args:tt ),+ $(,)?)) => {{
        $to_change.horizontal(|to_change| {
            $( layout!( @hint to_change $hint $hint_args ); )*
        });
    }};
    (@hint $to_change:ident rect ($refer:expr, $width:literal x $height:literal)) => {{
        *$refer = $to_change.pos();
        $to_change.add_rect($width, $height);
    }};
    ( $( $lay:ident $lay_args:tt ),+ $(,)? ) => {{
        let mut x = 0;
        let mut y = 0;
        let mut to_change = ToChange { x: &mut x, y: &mut y, axis: Axis::X};
        $( layout!( @hint to_change $lay $lay_args ); )*
    }};
}
struct ShipButtons {
    paladin: Pos,
    spear: Pos,
    blank: Pos,
}
impl ShipButtons {
    // TODO: automatic derive of a const_DEFAULT (or use existing crate)
    const DEFAULT: Self = Self {
        paladin: Pos::DEFAULT,
        spear: Pos::DEFAULT,
        blank: Pos::DEFAULT,
    };
}
struct MainButtons {
    start_game: Pos,
    ships: Pos,
}
impl MainButtons {
    // TODO: automatic derive of a const_DEFAULT (or use existing crate)
    const DEFAULT: Self = Self {
        ships: Pos::DEFAULT,
        start_game: Pos::DEFAULT,
    };
}
fn init_menu() {
    let mut image_ref = Pos::DEFAULT;
    let mut text_ref = Pos::DEFAULT;
    let mut ship_buttons = ShipButtons::DEFAULT;
    let mut main_buttons = MainButtons::DEFAULT;
    layout! {
        vertical(
            space(2),
            text("Select your ship:"),
            space(1),
            horizontal(
                select(&mut ship_buttons.blank, "Blank"),
                space(5),
                select(&mut ship_buttons.spear, "Spear"),
                space(5),
                select(&mut ship_buttons.paladin, "Paladin"),
            ),
            space(2),
            text("Current ship:"),
            space(1),
            horizontal(
                rect(&mut image_ref, 3 x 3),
                space(1),
                rect(&mut text_ref, 20 x 3),
            ),
        )
    };
    layout! {
        vertical(
            space(5),
            select(&mut main_buttons.start_game, "BEGIN GAME !!!"),
            space(2),
            select(&mut main_buttons.ships, "Ship select"),
        )
    };
}
