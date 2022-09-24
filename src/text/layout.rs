//! Manage text layout on screen.

use hal::video::{
    mode,
    tile::{map::Pos, sbb, Drawable},
};

#[doc(hidden)]
#[derive(Copy, Clone)]
pub(crate) enum Axis {
    X,
    Y,
}
#[doc(hidden)]
pub(crate) struct ToChange<'a, 'b, M: mode::TileMode> {
    pub(crate) axis: Axis,
    pub(crate) x: &'a mut usize,
    pub(crate) y: &'a mut usize,
    pub(crate) sbb: sbb::Handle<'b, M>,
}
/// State of layouting
impl<'a, 'b, M: mode::TileMode> ToChange<'a, 'b, M> {
    pub(crate) const fn current_axis(&mut self) -> &mut usize {
        match self.axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }
    pub(crate) const fn add_rect(&mut self, width: usize, height: usize) {
        let value = match self.axis {
            Axis::X => width,
            Axis::Y => height,
        };
        let to_change = self.current_axis();
        *to_change += value;
    }
    pub(crate) const fn add(&mut self, value: usize) {
        let to_change = self.current_axis();
        *to_change += value;
    }
    pub(crate) const fn pos(&self) -> Pos {
        Pos {
            x: *self.x,
            y: *self.y,
        }
    }
    pub(crate) fn vertical(&mut self, f: impl FnOnce(&mut Self)) {
        let old_axis = self.axis;
        let old_y = *self.y;
        self.axis = Axis::Y;
        f(self);
        self.axis = old_axis;
        *self.y = old_y;
    }
    pub(crate) fn horizontal(&mut self, f: impl FnOnce(&mut Self)) {
        let old_axis = self.axis;
        let old_x = *self.x;
        self.axis = Axis::X;
        f(self);
        self.axis = old_axis;
        *self.x = old_x;
    }
    pub(crate) fn draw(&mut self, drawable: &impl Drawable) {
        self.sbb.set_tiles(self.pos(), drawable);
    }
}

/// Draw a menu layout using a very barebone layouting specification.
///
/// # Syntax
///
/// ```text
/// layout!(
///     #[sbb($sbb: sbb::Handle)]
///     [<command>, ]*
/// )
/// ```
///
/// `$sbb` is the Screen block (tile map) to draw the screen to,
/// `[<command>, ]*` is a series of layout commands. There are three
/// kinds of commands:
/// - **context change** (`vertical`, `horizontal`): whether to advance
///   the cursor forward vertically or horizontally after drawing
///   something on screen.
/// - **drawing** (`space`, `text`, `image`): reserve and optionally draw
///   tiles on screen at the current cursor positions, then the cursor
///   is updated to a free screen region for the next command.
/// - **position save** (`select`, `rect`): act like **drawing** commands,
///   but also accepts a `&mut Pos` argument to save somewhere the current
///   cursor position. This let you keep track of screen regions so that
///   you can later manipulate them freely.
///
/// All the commands:
/// - **Context change**:
///     - `vertical( [<command>,]* )`: Enter "vertical" mode and execute specified
///       commands, return in current mode afterward.
///     - `horizontal( [<command>,]* )`: Enter "horizontal" mode and execute
///       specified commands, return in current mode afterward.
/// - **Drawing**:
///     - `space($count) `:  skip `$count` cells in current direction.
///     - `text($text: &'static str)`:  draw `$text` and advance `$text.len()`
///       cells accordingly.
///     - `image($img)`:  draw `$img` (a `Drawable`) and advance cells.
/// - **Position Save**:
///     - `select($ref: &mut Pos, $text: &'static str)`: draw `$text`, advance
///       `$text.len()` cells and save text position in `$ref`.
///     - `rect($ref: &mut Pos, $width x $height)`: Like `image`, but draws nothing,
///       just "reserves" a square of size `$width x $height` and saves the cursor
///       position in `$ref`.
///
/// # Example
///
/// ```
/// # use hal::{VideoControl, video::{self, tile::sbb}};
/// # let mut ctrl = unsafe { VideoControl::<video::mode::Text>::init() };
/// layout! {
///     #[sbb(ctrl.sbb(sbb::Slot::_15))]
///     vertical(
///         space(5),
///         image(crate::assets::menu::title_card),
///         space(1),
///         text("Press Start"),
///     )
/// };
/// ```
#[macro_export]
macro_rules! layout {
    (@hint $to_change:ident space ($count:literal)) => {
        $to_change.add($count)
    };
    (@hint $to_change:ident text ($text:expr)) => {
        $to_change.draw(&$text);
        let text_width = $text.split('\n').map(|line| line.len()).max().unwrap_or(0);
        let text_height = $text.chars().filter(|char| *char ==  '\n').count() + 1;
        $to_change.add_rect(text_width, text_height)
    };
    (@hint $to_change:ident image ($img:expr)) => {
        $to_change.draw(&$img);
        $to_change.add_rect($img.width, $img.height)
    };
    (@hint $to_change:ident select ($refer:expr, $text:expr)) => {
        $to_change.draw(&$text);
        *$refer = $to_change.pos();
        let text_width = $text.split('\n').map(|line| line.len()).max().unwrap_or(0);
        let text_height = $text.chars().filter(|char| *char ==  '\n').count() + 1;
        $to_change.add_rect(text_width, text_height)
    };
    (@hint $to_change:ident vertical ($( $hint:ident $hint_args:tt ),+ $(,)?)) => {
        $to_change.vertical(|to_change| {
            $( layout!( @hint to_change $hint $hint_args ); )*
        })
    };
    (@hint $to_change:ident horizontal ($( $hint:ident $hint_args:tt ),+ $(,)?)) => {
        $to_change.horizontal(|to_change| {
            $( layout!( @hint to_change $hint $hint_args ); )*
        })
    };
    (@hint $to_change:ident rect ($refer:expr, $width:tt x $height:tt)) => {
        *$refer = $to_change.pos();
        $to_change.add_rect($width, $height)
    };
    ( #[sbb($sbb:expr)] $( $lay:ident $lay_args:tt ),+ $(,)? ) => {{
        #[allow(unused)]
        use hal::video::tile::Drawable;
        use $crate::text::layout::{ToChange, Axis};
        let mut x = 0;
        let mut y = 0;
        let mut to_change = ToChange { x: &mut x, y: &mut y, axis: Axis::X, sbb: $sbb};
        $( layout!( @hint to_change $lay $lay_args ); )*
    }};
}
