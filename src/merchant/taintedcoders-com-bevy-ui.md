---
source: https://taintedcoders.com/bevy/ui
---

Bevy's UI is built completely within its ECS system.

We create UI elements by spawning entities with certain components that the UI part of the renderer cares about.

The most important of these components being the Node that controls a specific element's layout. These Nodes lay themselves out according to a specific set of rules that keeps their position relative to their parent and siblings.

Now, you might be thinking:

ECS works best when we know the components can be processed independently of each other. In a UI context the components are tightly coupled and usually do a lot of async event handling.

However, by keeping the UI inside of the ECS, we stand to gain some benefits:

Keeping consistent with the ECS data, no transfers to a virtual world
Able to build off of Bevy's other successes
Do more than draw boxes on a screen
Avoid problems with the borrow checker and graphs of UI elements
For a deeper discussion about these trade-offs I highly suggest reading Alice's post. Alice is the lead maintainer at Bevy.

Layout in Bevy is powered by the taffy library which is a high performance rust-powered UI layout library.

The bevy_ui crate uses an implementation of the Flexbox and CSS-Grid layout models. If you are familiar with web development, you will feel right at home.

A hierarchy of Node components are passed to a special render pipeline that is usually independent of our Camera viewport. So our UI stays put as we move the camera around and change what we are looking at.

These Node elements can be cumbersome to assemble by hand. Bevy now ships first-party widgets to make rich interactive UIs easier, without any extra dependencies:

bevy_ui_widgets provides us with buttons, checkboxes, sliders, radio groups, scrollbars, menus, popovers, lists, tabs, and text inputs
bevy_feathers is a larger styled and themed widget set aimed at editor tools. It requires enabling the bevy_feathers feature
There are also supporting third party libraries:

bevy_egui
bevy_lunex
State of the art
The UI libraries are in a constant state of flux. In August, 2023 Cart posted about the direction he wants to take Bevy's UI.

Bevy has been moving towards some kind of reactive framework for their UI and so while that settles it's kind of the wild west. Observers were a big step towards supporting the eventual architecture Cart wants.

As of 0.17 there is an official set of built-in Bevy components being made available through bevy_feathers, which was considered experimental until 0.19. These components are styled and themed widgets for building editors and inspectors.

Feathers is not enabled by default though, we must opt in with the bevy_feathers feature. The smaller bevy_ui_widgets crate is enabled by default as part of the ui collection.

0.19 introduced BSN (Bevy Scene Notation), Bevy's next generation scene system (covered in its own BSN guide), which is now the recommended way to declare UI.

The bsn! macro lets you write UI as a Rust-like scene instead of a long chain of commands.spawn(...) calls, with optional fields, Children [...] relationships, scene functions, and inline observers:

// The same UI can be declared with BSN (Bevy Scene Notation).
// This scene function can be spawned with `.spawn()` or `commands.spawn_scene`.
fn button_scene() -> impl Scene {
  bsn! {
    Button
    Node {
      width: px(150),
      height: px(65),
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center,
    }
    BackgroundColor(Color::BLACK)
    Children [
      Text::new("Button")
      TextColor(Color::srgb(0.9, 0.9, 0.9))
    ]
  }
}
Feathers widgets are moving to BSN as their primary definition format, and the picking example below already uses it.

If you're looking for early access to a next generation editor you can check out jackdaw as a preview of what's coming.

Drawing boxes
To draw a box on the screen as part of our UI all we need to do is use a Node component that determines how to lay itself out within a particular Window.

The basic mental model you should have is this:

A Window represents a space on your monitor
A Camera represents a region on a Window that it has a right to draw on
A Node represents a layout on the Window, on top of any Camera region
This is obviously not the full story. You can track the UI to a specific camera using the UiTargetCamera component for example. But it is a solid model for the default state of things.

Bevy calculates the sizes and layout of these components using a representation of your elements mirrored from the taffy crate.

Nodes are components that hold layout information. Nodes that are children of other nodes lay themselves out relative to their parent. To represent the hierarchy of our nodes, Bevy uses the ChildOf and Children components:

fn spawn_box(mut commands: Commands) {
  let container = Node {
    width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    justify_content: JustifyContent::Center,
    ..default()
  };

  let square = (
    BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
    Node {
      width: Val::Px(200.),
      border: UiRect::all(Val::Px(2.)),
      ..default()
    },
  );

  commands.spawn((container, children![(square)]));
}
Bevy has helper functions for Val to make things easier to construct. So we could have written them using the auto, px, percent, vw, vh, vmin, vmax, em or rem helpers:

let container = Node {
  width: percent(100.0),
  height: percent(100.0),
  justify_content: JustifyContent::Center,
  ..default()
};
We spawned an outer box and our child was placed inside it according to the layout rules of the parent.

Things in your game world have a Transform and GlobalTransform which tells the renderer where to put them on to the screen. Nodes use a corresponding set of UiTransform and UiGlobalTransform components.

All Children of a node will set their UiTransform to be relative to their parent, so the Node we spawned as a child will be placed in the center of its parent.

Even though your nodes have UiTransform and UiGlobalTransform you should never modify them directly. Always set your node's position through the layout and style properties inside the Node.

Displaying some text
Text can be rendered in two separate ways:

As part of our game with Text2d
As part of our UI with Text
So, to spawn text within our UI we use Text:

fn spawn_text_in_ui(mut commands: Commands) {
  commands.spawn((
    Node {
      position_type: PositionType::Absolute,
      bottom: px(5.0),
      right: px(5.0),
      ..default()
    },
    Text::new("Here is some text"),
    TextColor(Color::BLACK),
    TextLayout::justify(Justify::Center),
  ));
}
The text we spawned would stay fixed to the screen. A Text component requires Node, as well as some additional text property components like alignment.

fn spawn_text_in_world(mut commands: Commands) {
  commands.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    Text2d::new("Here is some text"),
    TextColor(Color::BLACK),
    TextLayout::justify(Justify::Center),
  ));
}
The text spawned here would instead be determined by the Camera. If we moved the camera around, the text would move with it. We control its position via a Transform rather than a Node.

Color
All colors are created from a certain color space.

Each type of color space has its own dedicated struct. For example your standard RGBA looks like this:

struct Srgba {
  red: f32,
  green: f32,
  blue: f32,
  alpha: f32,
}
A Color is an enum over all the different color spaces, each with a separate implementation.

We can easily convert from one type to another using the From and Into traits:

let color: Hsla = Srgba::rgb(1.0, 0.0, 1.0).into();
It's very common to use the helper methods on the Color enum:

let black = Color::srgb(0.0, 0.0, 0.0);
let red = Color::hsv(0., 1., 1.);
If we want to convert from one color to another we need to convert our color into the desired space and then perform our action before converting back:

let black = Color::srgb(0.0, 0.0, 0.0);
let srgba = Srgba {
  blue: 1.0,
  ..Srgba::from(black),
};
let blue = Color::from(srgba);
To change the transparency we can use the methods from the Alpha trait:

Color::set_alpha
Color::with_alpha
Color::alpha
Color::is_fully_transparent
There are a convenient set of color constants available inside the palettes::css namespace:

use bevy::color::palettes::css::{BLACK, BLUE, WHITE};
These can be very useful for prototyping out your game without worrying about the specific color spaces.

Reacting to a button press
Typically you would use Bevy's built-in picking features to react to button presses. This is done through observers:

use bevy::color::palettes::basic::GREEN;

fn change_button_color_on_hover(
  hover: On<PointerOver>,
  mut commands: Commands,
) {
  commands
    .entity(hover.entity)
    .insert(BackgroundColor(GREEN.into()));
}

fn button() -> impl Scene {
  bsn! {
    Button
    Node {
      width: px(150),
      height: px(65),
      border: UiRect::all(px(5)),
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center
    }
    BorderColor::all(Color::WHITE)
    BackgroundColor(Color::BLACK)
    Children [
      Text::new("Button")
      TextColor(Color::srgb(0.9, 0.9, 0.9))
      TextShadow::default()
    ]
  }
}

fn layout() -> impl Scene {
  bsn! {
    Node {
      width: percent(100),
      height: percent(100),
      align_items: AlignItems::Center,
      justify_content: JustifyContent::Center,
    }
    Children [
      @button()
      on(change_button_color_on_hover)
    ]
  }
}
Underneath, the picking backend tracks hover and press state with the Hovered and Pressed components. We can query those directly in our own systems:

use bevy::color::palettes::css::*;

fn button_system(
  mut interactions: Query<
    (
      &Hovered,
      Has<Pressed>,
      &mut BackgroundColor,
      &mut BorderColor,
      &Children,
    ),
    With<Button>,
  >,
  mut texts: Query<&mut Text>,
) {
  for (hovered, pressed, mut color, mut border_color, children) in
    &mut interactions
  {
    if let Ok(mut text) = texts.get_mut(children[0]) {
      match (hovered.get(), pressed) {
        (_, true) => {
          text.0 = "Press".to_string();
          *color = PRESSED_BUTTON.into();
          border_color.set_all(BLUE);
        }
        (true, false) => {
          text.0 = "Hover".to_string();
          *color = HOVERED_BUTTON.into();
          border_color.set_all(WHITE);
        }
        (false, false) => {
          text.0 = "Button".to_string();
          *color = NORMAL_BUTTON.into();
          border_color.set_all(BLACK);
        }
      }
    }
  }
}
Depending on whether we hover, press or do nothing with the button, the background color and text will change.

Widgets
The bevy_ui_widgets crate provides basic interactive controls:

Buttons
Checkboxes
Radio groups
Sliders
Scrollbars
Menus
Popovers
Lists
Tabs
Text inputs
These widgets are headless, they manage interaction, focus, and accessibility for you but bring no styling of their own.

We can spawn the widget component and attach an observer to react to its events. A bevy_ui_widgets::Button emits an Activate event when it is released:

use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

// `bevy_ui_widgets` widgets are headless: they handle interaction, focus, and
// accessibility, but leave all styling up to us.
fn spawn_widget_button(mut commands: Commands) {
  commands
    .spawn((
      Button,
      Node {
        width: px(150),
        height: px(65),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
      },
      BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
      children![(Text::new("Click me"), TextColor(Color::WHITE))],
    ))
    .observe(|_activate: On<Activate>| {
      info!("Button clicked!");
    });
}
Notice that here we imported bevy_ui_widgets::Button explicitly. Bevy's prelude also exposes a simpler Button marker (bevy::ui::widget::Button) that draws a box and reports pointer interaction, but as of 0.20 that marker is deprecated in favor of bevy_ui_widgets::Button. The widget version adds keyboard activation, accessibility, and the Activate event on top.

bevy_feathers is a larger, styled widget set built on top of bevy_ui_widgets. It requires enabling the bevy_feathers feature and adds a theme system. Install the plugin and a theme once at startup:

use bevy::prelude::*;
use bevy::feathers::{dark_theme::create_dark_theme, theme::UiTheme, FeathersPlugins};

// Feathers needs its plugin and a theme installed once at startup.
fn feathers_app() {
  App::new()
    .add_plugins((DefaultPlugins, FeathersPlugins))
    .insert_resource(UiTheme(create_dark_theme()))
    .add_systems(Startup, feathers_scene.spawn())
    .run();
}
Feathers widgets are defined with BSN, so we reference them with the @ scene-component syntax and attach observers inline:

use bevy::prelude::*;
use bevy::feathers::{controls::FeathersButton, theme::{ThemeBackgroundColor, ThemedText}, tokens};
use bevy::ui_widgets::Activate;

// Feathers widgets are BSN scene components, referenced with the `@` syntax.
fn feathers_scene() -> impl Scene {
  bsn! {
    Node {
      width: percent(100),
      height: percent(100),
      align_items: AlignItems::Center,
      justify_content: JustifyContent::Center,
    }
    ThemeBackgroundColor(tokens::WINDOW_BG)
    Children [
      @FeathersButton
      on(|_activate: On<Activate>| {
        info!("Decrement");
      })
      Children [ Text("-1") ThemedText ]
      --
      @FeathersButton
      on(|_activate: On<Activate>| {
        info!("Increment");
      })
      Children [ Text("+1") ThemedText ]
    ]
  }
}
Feathers theming is contextual. Widgets pull their colors from SemanticTokens resolved through a ThemeContext, and a custom ThemeProps maps those semantic tokens to your palette.

Nodes
A Node is a component that holds the layout and style properties.

Nodes are laid out with either a flexbox or CSS grid layout.

Whether or not your UI follows the Camera can be controlled by adding a UiTargetCamera component to your Node based entity. Which camera renders UI by default is controlled by the DefaultUiCamera component. Each node then resolves its target through the ComputedUiTargetCamera component that Bevy propagates down the hierarchy.

A Node also requires a number of supporting components. The most important is ComputedNode, which contains the actual calculated size and metrics of your node. The renderer uses this to lay the node out on the screen.

This is what the default style of a Node looks like:

// https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_ui/src/ui_node.rs#L815
impl Node {
  pub const DEFAULT: Self = Self {
    display: Display::DEFAULT,
    box_sizing: BoxSizing::DEFAULT,
    position_type: PositionType::DEFAULT,
    left: Val::Auto,
    right: Val::Auto,
    top: Val::Auto,
    bottom: Val::Auto,
    flex_direction: FlexDirection::DEFAULT,
    flex_wrap: FlexWrap::DEFAULT,
    align_items: AlignItems::DEFAULT,
    justify_items: JustifyItems::DEFAULT,
    align_self: AlignSelf::DEFAULT,
    justify_self: JustifySelf::DEFAULT,
    align_content: AlignContent::DEFAULT,
    justify_content: JustifyContent::DEFAULT,
    direction: InlineDirection::Ltr,
    margin: UiRect::DEFAULT,
    padding: UiRect::DEFAULT,
    border: UiRect::DEFAULT,
    border_radius: BorderRadius::DEFAULT,
    flex_grow: 0.0,
    flex_shrink: 1.0,
    flex_basis: Val::Auto,
    width: Val::Auto,
    height: Val::Auto,
    min_width: Val::Auto,
    min_height: Val::Auto,
    max_width: Val::Auto,
    max_height: Val::Auto,
    aspect_ratio: None,
    overflow: Overflow::DEFAULT,
    overflow_clip_margin: OverflowClipMargin::DEFAULT,
    scrollbar_width: 0.,
    row_gap: Val::ZERO,
    column_gap: Val::ZERO,
    grid_auto_flow: GridAutoFlow::DEFAULT,
    grid_template_rows: Vec::new(),
    grid_template_columns: Vec::new(),
    grid_auto_rows: Vec::new(),
    grid_auto_columns: Vec::new(),
    grid_column: GridPlacement::DEFAULT,
    grid_row: GridPlacement::DEFAULT,
  };
}
It holds all the normal CSS styles we use to automatically lay them out relative to each other.

The Val items are an enum representing the different measurements we can use for our value:

Type	Description
Val::Auto	Automatically determine the value based on the context and other properties
Val::Px(f32)	Pixel based values
Val::Percent(f32)	Percentage based along a specific axis which depends on other fields
Val::Vw(f32)	Percentage of the viewport width
Val::Vh(f32)	Percentage of the viewport height
Val::VMin(f32)	Percentage of the viewport's smaller dimension
Val::VMax(f32)	Percentage of the viewport's larger dimension
Val::Em(f32)	Multiple of the node's own font size (EmSize)
Val::Rem(f32)	Multiple of the root font size (RemSize)
Fixed-size values are affected by the UiScale resource. It only scales logical-pixel (Val::Px) values by the multiplier you set; percentage and viewport-relative units are not scaled.

Margin, padding and borders are implemented using UiRect.

Rendering order
UI work is split across a set of system sets defined by the UiSystems enum. Focus runs in PreUpdate, while the rest are chained together in PostUpdate:

Focus (PreUpdate): Where we handle input interactions with UI entities
Prepare: Propagate camera targets and other data needed by layout
Propagate: Propagate UI component values needed by layout
Content: Update content requirements (such as text and image sizes) before layout
Layout: Where we update the layout state of the UI
PostLayout: Systems ordered after layout, such as clipping updates
Stack: Where we update the UiStack resource, ordering UI nodes by their depth
The UiStack orders the UI nodes so that we can have stacking windows. The first entry is the furthest node and the first to get rendered on the screen.

However the first node is also the last to receive any interactions so it's actually the final node that would be interacted with.

// https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_ui/src/stack.rs#L30

// The first entry is the furthest node from the camera
// and is the first one to get rendered, while the last
// entry is the first node to receive interactions.
#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct UiStack {
  /// Partition of the `uinodes` list into disjoint slices of nodes that all share the same camera target.
  pub partition: Vec<Range<usize>>,
  /// List of UI nodes ordered from back-to-front
  pub uinodes: Vec<Entity>,
}
We generate this stack by walking the UI hierarchy, sorting siblings by their local ZIndex. The local ZIndex only orders a node relative to its siblings.

A GlobalZIndex is different. It lets a node escape the implicit draw ordering of the layout tree and be drawn above or below other UI nodes.

Relative Cursor Position
A Node's UiTransform is not directly related to its actual position on screen. To manually calculate the cursor's position relative to a Node we would also need the Window, the node's ComputedNode, its UiGlobalTransform, the target Camera, and the CalculatedClip.

To help make this easier, we can spawn a RelativeCursorPosition component on each entity we care about that has a Node component. Bevy will keep this updated automatically.

The RelativeCursorPosition component stores the cursor position relative to our node. If it is within the range of (-0.5, -0.5) to (0.5, 0.5) then the cursor is currently over the node, with (0., 0.) being center. You can use the cursor_over: bool field to figure this out.

If the cursor position is unknown (e.g we are alt+tabbed out of our game) then the position will be None.

We can query this component in our systems to get this info:

use bevy::ui::RelativeCursorPosition;
// This systems polls the relative cursor position
// and displays its value in a text component.
fn relative_cursor_position(cursor_query: Query<&RelativeCursorPosition>) {
  if let Ok(cursor) = cursor_query.single() {
    // This is an `Option<Vec2>` as an unknown cursor position
    // will return `None`
    if let Some(cursor) = cursor.normalized {
      info!("({:.1}, {:.1})", cursor.x, cursor.y)
    }
  }
}
Interaction
To interact with the UI we should use Bevy's built-in picking features.

Scrolling
A Node that has an axis set to OverflowAxis::Scroll will reserve space for a scrollbar according to the node's scrollbar_width field. There are convenience constructors such as Overflow::scroll_y() and Overflow::scroll_x().

fn spawn_scrollable_box(mut commands: Commands) {
  let container = Node {
    width: percent(100.0),
    height: percent(100.0),
    justify_content: JustifyContent::Center,
    ..default()
  };
  let scrollable = (
    BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
    Node {
      width: px(200.),
      height: px(200.),
      border: UiRect::all(px(2.)),
      overflow: Overflow::scroll_y(),
      ..default()
    },
  );
  commands.spawn((container, children![(scrollable)]));
}
Read more
https://hackmd.io/@bevy/editor_questions
https://deadmoney.gg/news/articles/how-do-nice-ui-in-bevy
https://www.leafwing-studios.com/blog/ecs-gui-framework/
https://github.com/bevyengine/bevy/blob/main/crates/bevy_ui/src/lib.rs
https://github.com/bevyengine/bevy/discussions/8941
https://www.gamedev.net/forums/topic/680812-ecs-gui-systems/
https://github.com/bevyengine/bevy/blob/main/examples/ui/widgets/butt