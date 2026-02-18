use bevy::prelude::*;
use kb_hall::AnalogKeyboard;

const VID: u16 = 0x41e4;
const PID: u16 = 0x2103;

const LAYOUT: &[&[(u8, &str, f32)]] = &[
    &[
        (0x29, "Esc", 1.0),
        (0x1E, "1", 1.0),
        (0x1F, "2", 1.0),
        (0x20, "3", 1.0),
        (0x21, "4", 1.0),
        (0x22, "5", 1.0),
        (0x23, "6", 1.0),
        (0x24, "7", 1.0),
        (0x25, "8", 1.0),
        (0x26, "9", 1.0),
        (0x27, "0", 1.0),
        (0x2D, "-", 1.0),
        (0x2E, "=", 1.0),
        (0x2A, "Bksp", 2.0),
    ],
    &[
        (0x2B, "Tab", 1.5),
        (0x14, "Q", 1.0),
        (0x1A, "W", 1.0),
        (0x08, "E", 1.0),
        (0x15, "R", 1.0),
        (0x17, "T", 1.0),
        (0x1C, "Y", 1.0),
        (0x18, "U", 1.0),
        (0x0C, "I", 1.0),
        (0x12, "O", 1.0),
        (0x13, "P", 1.0),
        (0x2F, "[", 1.0),
        (0x30, "]", 1.0),
        (0x31, "\\", 1.5),
    ],
    &[
        (0x39, "Caps", 1.75),
        (0x04, "A", 1.0),
        (0x16, "S", 1.0),
        (0x07, "D", 1.0),
        (0x09, "F", 1.0),
        (0x0A, "G", 1.0),
        (0x0B, "H", 1.0),
        (0x0D, "J", 1.0),
        (0x0E, "K", 1.0),
        (0x0F, "L", 1.0),
        (0x33, ";", 1.0),
        (0x34, "'", 1.0),
        (0x28, "Enter", 2.25),
    ],
    &[
        (0xE1, "Shift", 2.25),
        (0x1D, "Z", 1.0),
        (0x1B, "X", 1.0),
        (0x06, "C", 1.0),
        (0x19, "V", 1.0),
        (0x05, "B", 1.0),
        (0x11, "N", 1.0),
        (0x10, "M", 1.0),
        (0x36, ",", 1.0),
        (0x37, ".", 1.0),
        (0x38, "/", 1.0),
        (0xE5, "Shift", 2.75),
    ],
    &[
        (0xE0, "Ctrl", 1.25),
        (0xE3, "Win", 1.25),
        (0xE2, "Alt", 1.25),
        (0x2C, "Space", 6.25),
        (0xE6, "Alt", 1.25),
        (0xFF, "Fn", 1.25),
        (0x65, "Menu", 1.25),
        (0xE4, "Ctrl", 1.25),
    ],
];

const KEY_UNIT: f32 = 46.0;
const KEY_H: f32 = 42.0;
const KEY_GAP: f32 = 4.0;
const ROW_W: f32 = 15.0;

#[derive(Resource)]
struct AppState {
    kb: AnalogKeyboard,
    display: [f32; 256],
}

#[derive(Component)]
struct Cap(u8);
#[derive(Component)]
struct Fill(u8);
#[derive(Component)]
struct Lbl;
#[derive(Component)]
struct StatusTxt;
#[derive(Component)]
struct PctTxt(u8);

fn main() {
    let kb = AnalogKeyboard::new(VID, PID);
    kb.start();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "KB Hall".into(),
                resolution: (900.0, 420.0).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.09, 0.09, 0.09)))
        .insert_resource(AppState {
            kb,
            display: [0.0; 256],
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (read_bevy_keys, animate_values, update_vis, update_hud).chain(),
        )
        .run();
}

fn keycode_to_sc(k: KeyCode) -> Option<u8> {
    Some(match k {
        KeyCode::KeyA => 0x04,
        KeyCode::KeyB => 0x05,
        KeyCode::KeyC => 0x06,
        KeyCode::KeyD => 0x07,
        KeyCode::KeyE => 0x08,
        KeyCode::KeyF => 0x09,
        KeyCode::KeyG => 0x0A,
        KeyCode::KeyH => 0x0B,
        KeyCode::KeyI => 0x0C,
        KeyCode::KeyJ => 0x0D,
        KeyCode::KeyK => 0x0E,
        KeyCode::KeyL => 0x0F,
        KeyCode::KeyM => 0x10,
        KeyCode::KeyN => 0x11,
        KeyCode::KeyO => 0x12,
        KeyCode::KeyP => 0x13,
        KeyCode::KeyQ => 0x14,
        KeyCode::KeyR => 0x15,
        KeyCode::KeyS => 0x16,
        KeyCode::KeyT => 0x17,
        KeyCode::KeyU => 0x18,
        KeyCode::KeyV => 0x19,
        KeyCode::KeyW => 0x1A,
        KeyCode::KeyX => 0x1B,
        KeyCode::KeyY => 0x1C,
        KeyCode::KeyZ => 0x1D,
        KeyCode::Digit1 => 0x1E,
        KeyCode::Digit2 => 0x1F,
        KeyCode::Digit3 => 0x20,
        KeyCode::Digit4 => 0x21,
        KeyCode::Digit5 => 0x22,
        KeyCode::Digit6 => 0x23,
        KeyCode::Digit7 => 0x24,
        KeyCode::Digit8 => 0x25,
        KeyCode::Digit9 => 0x26,
        KeyCode::Digit0 => 0x27,
        KeyCode::Enter => 0x28,
        KeyCode::Escape => 0x29,
        KeyCode::Backspace => 0x2A,
        KeyCode::Tab => 0x2B,
        KeyCode::Space => 0x2C,
        KeyCode::Minus => 0x2D,
        KeyCode::Equal => 0x2E,
        KeyCode::BracketLeft => 0x2F,
        KeyCode::BracketRight => 0x30,
        KeyCode::Backslash => 0x31,
        KeyCode::Semicolon => 0x33,
        KeyCode::Quote => 0x34,
        KeyCode::Backquote => 0x35,
        KeyCode::Comma => 0x36,
        KeyCode::Period => 0x37,
        KeyCode::Slash => 0x38,
        KeyCode::CapsLock => 0x39,
        KeyCode::ShiftLeft => 0xE1,
        KeyCode::ShiftRight => 0xE5,
        KeyCode::ControlLeft => 0xE0,
        KeyCode::ControlRight => 0xE4,
        KeyCode::AltLeft => 0xE2,
        KeyCode::AltRight => 0xE6,
        KeyCode::SuperLeft => 0xE3,
        KeyCode::SuperRight => 0xE7,
        KeyCode::ContextMenu => 0x65,
        _ => return None,
    })
}

fn read_bevy_keys(keys: Res<ButtonInput<KeyCode>>, state: Res<AppState>) {
    if state.kb.is_active() {
        return;
    }
    let mut vals = [0.0f32; 256];
    for key in keys.get_pressed() {
        if let Some(sc) = keycode_to_sc(*key) {
            vals[sc as usize] = 1.0;
        }
    }
    state.kb.set_values(&vals);
}

fn animate_values(mut state: ResMut<AppState>, time: Res<Time>) {
    let dt = time.delta_secs();
    let target = state.kb.values();
    for i in 0..256 {
        let t = target[i];
        let d = &mut state.display[i];
        if t > *d {
            *d = (*d + 25.0 * dt).min(t);
        } else {
            *d = (*d - 8.0 * dt).max(t);
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let bw = ROW_W * (KEY_UNIT + KEY_GAP) - KEY_GAP;
    let bh = 5.0 * (KEY_H + KEY_GAP) - KEY_GAP;
    let ox = -bw / 2.0;
    let oy = bh / 2.0 + 20.0;

    for (ri, row) in LAYOUT.iter().enumerate() {
        let mut xo = 0.0f32;
        for &(sc, label, wu) in *row {
            let kw = wu * KEY_UNIT + (wu - 1.0) * KEY_GAP;
            let cx = ox + xo + kw / 2.0;
            let cy = oy - ri as f32 * (KEY_H + KEY_GAP) - KEY_H / 2.0;

            commands.spawn((
                Sprite {
                    color: Color::srgb(0.20, 0.20, 0.20),
                    custom_size: Some(Vec2::new(kw, KEY_H)),
                    ..default()
                },
                Transform::from_xyz(cx, cy, 0.0),
                Cap(sc),
            ));
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.7, 1.0),
                    custom_size: Some(Vec2::new(kw - 4.0, 0.0)),
                    anchor: bevy::sprite::Anchor::BottomCenter,
                    ..default()
                },
                Transform::from_xyz(cx, cy - KEY_H / 2.0 + 2.0, 1.0),
                Fill(sc),
            ));
            commands.spawn((
                Text2d::new(label),
                TextFont {
                    font_size: if wu > 1.5 { 10.0 } else { 13.0 },
                    ..default()
                },
                TextColor(Color::srgb(0.63, 0.63, 0.63)),
                Transform::from_xyz(cx, cy + 6.0, 2.0),
                Lbl,
            ));
            commands.spawn((
                Text2d::new(""),
                TextFont {
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                Transform::from_xyz(cx, cy - 8.0, 3.0),
                PctTxt(sc),
            ));
            xo += kw + KEY_GAP;
        }
    }

    commands.spawn((
        Text2d::new("Starting..."),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.53, 0.53, 0.53)),
        Transform::from_xyz(0.0, oy - bh - 18.0, 2.0),
        StatusTxt,
    ));
}

fn update_vis(
    state: Res<AppState>,
    mut fills: Query<(&Fill, &mut Sprite), Without<Cap>>,
    mut caps: Query<(&Cap, &mut Sprite), (Without<Fill>, Without<PctTxt>)>,
    mut pcts: Query<(&PctTxt, &mut Text2d, &mut TextColor)>,
) {
    let disp = &state.display;

    for (f, mut sp) in fills.iter_mut() {
        let v = disp[f.0 as usize].clamp(0.0, 1.0);
        let w = sp.custom_size.map(|s| s.x).unwrap_or(42.0);
        sp.custom_size = Some(Vec2::new(w, v * (KEY_H - 4.0)));
        let r = v * v;
        let g = 0.4 + 0.6 * (1.0 - (v - 0.5).abs() * 2.0).max(0.0);
        let b = 1.0 - v * 0.4;
        sp.color = Color::srgb(r, g, b);
    }

    for (c, mut sp) in caps.iter_mut() {
        let v = disp[c.0 as usize].clamp(0.0, 1.0);
        let g = v * 0.2;
        sp.color = Color::srgb(0.20 + g, 0.20 + g, 0.20 + g);
    }

    for (p, mut txt, mut col) in pcts.iter_mut() {
        let v = disp[p.0 as usize].clamp(0.0, 1.0);
        if v > 0.01 {
            let pct = (v * 100.0).round();
            **txt = format!("{pct:.0}%");
            *col = TextColor(Color::srgba(1.0, 1.0, 1.0, (v * 2.0).min(0.9)));
        } else {
            **txt = String::new();
            *col = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0));
        }
    }
}

fn update_hud(state: Res<AppState>, mut sq: Query<(&mut Text2d, &mut TextColor), With<StatusTxt>>) {
    let active = state.kb.is_active();
    let st = state.kb.status();

    for (mut t, mut c) in sq.iter_mut() {
        **t = st.clone();
        if active {
            *c = TextColor(Color::srgb(0.3, 0.9, 0.4));
        } else {
            *c = TextColor(Color::srgb(0.9, 0.7, 0.3));
        }
    }
}
