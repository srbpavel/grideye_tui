use crate::PALLETE;
use crate::ui::TEMPERATURE_INDEX_STEP;
use crate::mqtt::Temperature;

pub type ColorIndex = Vec<(Temperature, ColorRGB)>;
pub type ColorRGB = (u8, u8, u8);

#[allow(unused)]
enum Rgb {
    Red,
    Yellow,
    Green,
    Blue,
}

/*
//
//#[allow(unused)]
pub fn generate_pallete() -> Vec<(u8, u8, u8)> {
    let range_red = color_range(128, 255, 1, Rgb::Red);
    let range_yellow = color_range(128, 255, 1, Rgb::Yellow);
    let range_green = color_range(128, 255, 1, Rgb::Green);
    let range_blue = color_range(128, 255, 1, Rgb::Blue);

    vec![
        range_blue,   // cold -> dark to light
        range_green,  //      -> dark to light
        range_yellow, //      -> light to dark
        range_red,    // hot  -> light to dark
    ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
}
*/

/*
//
//#[allow(unused)]
fn color_range(start: u8,
               stop: u8,
               step: usize,
               color: Rgb) -> Vec<(u8, u8, u8)> {

    (start..stop)
        .into_iter()
        .step_by(step)
        .map(|c| {
            match color {
                Rgb::Red => (c, 0_u8, 0_u8),
                Rgb::Yellow => (255, c, 0_u8),
                Rgb::Green => (0_u8, c, 0_u8),
                Rgb::Blue => (0_u8, 0_u8, c),
            }
        })
        .collect::<Vec<_>>()
}
*/

//
//#[allow(unused)]
pub fn temperature_to_color(pallete: &[(f32, (u8, u8, u8))],
                            temperature: Temperature,
) -> Option<(u8, u8, u8)> {
    pallete
        .iter()
        .find_map(|(index, color)| {
            if (*index == temperature).eq(&true) {
                Some(*color)
            } else { None }
        })
}

//
pub fn index_color_pallete(boundary_max: Temperature,
                       boundary_min: Temperature,
) -> ColorIndex
{
    let range = (boundary_max - boundary_min) / PALLETE.len() as Temperature;
    
    PALLETE
        .iter()
        .enumerate()
        .map(|(index, color)| {
            // this keeps the lowest value inaccessible
            // so color will be COLOR_NONE_MAP
            // todo!() try harder
            let temperature = ((boundary_min as Temperature + (index + 1) as Temperature * range) / TEMPERATURE_INDEX_STEP).ceil() * TEMPERATURE_INDEX_STEP;
            
            (temperature, *color)
        })
        .collect::<ColorIndex>()
}

/*
//#[allow(unused)]
pub const PALLETE_RGB: [(u8, u8, u8); 381] = [(0, 0, 128), (0, 0, 129), (0, 0, 130), (0, 0, 131), (0, 0, 132), (0, 0, 133), (0, 0, 134), (0, 0, 135), (0, 0, 136), (0, 0, 137), (0, 0, 138), (0, 0, 139), (0, 0, 140), (0, 0, 141), (0, 0, 142), (0, 0, 143), (0, 0, 144), (0, 0, 145), (0, 0, 146), (0, 0, 147), (0, 0, 148), (0, 0, 149), (0, 0, 150), (0, 0, 151), (0, 0, 152), (0, 0, 153), (0, 0, 154), (0, 0, 155), (0, 0, 156), (0, 0, 157), (0, 0, 158), (0, 0, 159), (0, 0, 160), (0, 0, 161), (0, 0, 162), (0, 0, 163), (0, 0, 164), (0, 0, 165), (0, 0, 166), (0, 0, 167), (0, 0, 168), (0, 0, 169), (0, 0, 170), (0, 0, 171), (0, 0, 172), (0, 0, 173), (0, 0, 174), (0, 0, 175), (0, 0, 176), (0, 0, 177), (0, 0, 178), (0, 0, 179), (0, 0, 180), (0, 0, 181), (0, 0, 182), (0, 0, 183), (0, 0, 184), (0, 0, 185), (0, 0, 186), (0, 0, 187), (0, 0, 188), (0, 0, 189), (0, 0, 190), (0, 0, 191), (0, 0, 192), (0, 0, 193), (0, 0, 194), (0, 0, 195), (0, 0, 196), (0, 0, 197), (0, 0, 198), (0, 0, 199), (0, 0, 200), (0, 0, 201), (0, 0, 202), (0, 0, 203), (0, 0, 204), (0, 0, 205), (0, 0, 206), (0, 0, 207), (0, 0, 208), (0, 0, 209), (0, 0, 210), (0, 0, 211), (0, 0, 212), (0, 0, 213), (0, 0, 214), (0, 0, 215), (0, 0, 216), (0, 0, 217), (0, 0, 218), (0, 0, 219), (0, 0, 220), (0, 0, 221), (0, 0, 222), (0, 0, 223), (0, 0, 224), (0, 0, 225), (0, 0, 226), (0, 0, 227), (0, 0, 228), (0, 0, 229), (0, 0, 230), (0, 0, 231), (0, 0, 232), (0, 0, 233), (0, 0, 234), (0, 0, 235), (0, 0, 236), (0, 0, 237), (0, 0, 238), (0, 0, 239), (0, 0, 240), (0, 0, 241), (0, 0, 242), (0, 0, 243), (0, 0, 244), (0, 0, 245), (0, 0, 246), (0, 0, 247), (0, 0, 248), (0, 0, 249), (0, 0, 250), (0, 0, 251), (0, 0, 252), (0, 0, 253), (0, 0, 254), (0, 128, 0), (0, 129, 0), (0, 130, 0), (0, 131, 0), (0, 132, 0), (0, 133, 0), (0, 134, 0), (0, 135, 0), (0, 136, 0), (0, 137, 0), (0, 138, 0), (0, 139, 0), (0, 140, 0), (0, 141, 0), (0, 142, 0), (0, 143, 0), (0, 144, 0), (0, 145, 0), (0, 146, 0), (0, 147, 0), (0, 148, 0), (0, 149, 0), (0, 150, 0), (0, 151, 0), (0, 152, 0), (0, 153, 0), (0, 154, 0), (0, 155, 0), (0, 156, 0), (0, 157, 0), (0, 158, 0), (0, 159, 0), (0, 160, 0), (0, 161, 0), (0, 162, 0), (0, 163, 0), (0, 164, 0), (0, 165, 0), (0, 166, 0), (0, 167, 0), (0, 168, 0), (0, 169, 0), (0, 170, 0), (0, 171, 0), (0, 172, 0), (0, 173, 0), (0, 174, 0), (0, 175, 0), (0, 176, 0), (0, 177, 0), (0, 178, 0), (0, 179, 0), (0, 180, 0), (0, 181, 0), (0, 182, 0), (0, 183, 0), (0, 184, 0), (0, 185, 0), (0, 186, 0), (0, 187, 0), (0, 188, 0), (0, 189, 0), (0, 190, 0), (0, 191, 0), (0, 192, 0), (0, 193, 0), (0, 194, 0), (0, 195, 0), (0, 196, 0), (0, 197, 0), (0, 198, 0), (0, 199, 0), (0, 200, 0), (0, 201, 0), (0, 202, 0), (0, 203, 0), (0, 204, 0), (0, 205, 0), (0, 206, 0), (0, 207, 0), (0, 208, 0), (0, 209, 0), (0, 210, 0), (0, 211, 0), (0, 212, 0), (0, 213, 0), (0, 214, 0), (0, 215, 0), (0, 216, 0), (0, 217, 0), (0, 218, 0), (0, 219, 0), (0, 220, 0), (0, 221, 0), (0, 222, 0), (0, 223, 0), (0, 224, 0), (0, 225, 0), (0, 226, 0), (0, 227, 0), (0, 228, 0), (0, 229, 0), (0, 230, 0), (0, 231, 0), (0, 232, 0), (0, 233, 0), (0, 234, 0), (0, 235, 0), (0, 236, 0), (0, 237, 0), (0, 238, 0), (0, 239, 0), (0, 240, 0), (0, 241, 0), (0, 242, 0), (0, 243, 0), (0, 244, 0), (0, 245, 0), (0, 246, 0), (0, 247, 0), (0, 248, 0), (0, 249, 0), (0, 250, 0), (0, 251, 0), (0, 252, 0), (0, 253, 0), (0, 254, 0), (128, 0, 0), (129, 0, 0), (130, 0, 0), (131, 0, 0), (132, 0, 0), (133, 0, 0), (134, 0, 0), (135, 0, 0), (136, 0, 0), (137, 0, 0), (138, 0, 0), (139, 0, 0), (140, 0, 0), (141, 0, 0), (142, 0, 0), (143, 0, 0), (144, 0, 0), (145, 0, 0), (146, 0, 0), (147, 0, 0), (148, 0, 0), (149, 0, 0), (150, 0, 0), (151, 0, 0), (152, 0, 0), (153, 0, 0), (154, 0, 0), (155, 0, 0), (156, 0, 0), (157, 0, 0), (158, 0, 0), (159, 0, 0), (160, 0, 0), (161, 0, 0), (162, 0, 0), (163, 0, 0), (164, 0, 0), (165, 0, 0), (166, 0, 0), (167, 0, 0), (168, 0, 0), (169, 0, 0), (170, 0, 0), (171, 0, 0), (172, 0, 0), (173, 0, 0), (174, 0, 0), (175, 0, 0), (176, 0, 0), (177, 0, 0), (178, 0, 0), (179, 0, 0), (180, 0, 0), (181, 0, 0), (182, 0, 0), (183, 0, 0), (184, 0, 0), (185, 0, 0), (186, 0, 0), (187, 0, 0), (188, 0, 0), (189, 0, 0), (190, 0, 0), (191, 0, 0), (192, 0, 0), (193, 0, 0), (194, 0, 0), (195, 0, 0), (196, 0, 0), (197, 0, 0), (198, 0, 0), (199, 0, 0), (200, 0, 0), (201, 0, 0), (202, 0, 0), (203, 0, 0), (204, 0, 0), (205, 0, 0), (206, 0, 0), (207, 0, 0), (208, 0, 0), (209, 0, 0), (210, 0, 0), (211, 0, 0), (212, 0, 0), (213, 0, 0), (214, 0, 0), (215, 0, 0), (216, 0, 0), (217, 0, 0), (218, 0, 0), (219, 0, 0), (220, 0, 0), (221, 0, 0), (222, 0, 0), (223, 0, 0), (224, 0, 0), (225, 0, 0), (226, 0, 0), (227, 0, 0), (228, 0, 0), (229, 0, 0), (230, 0, 0), (231, 0, 0), (232, 0, 0), (233, 0, 0), (234, 0, 0), (235, 0, 0), (236, 0, 0), (237, 0, 0), (238, 0, 0), (239, 0, 0), (240, 0, 0), (241, 0, 0), (242, 0, 0), (243, 0, 0), (244, 0, 0), (245, 0, 0), (246, 0, 0), (247, 0, 0), (248, 0, 0), (249, 0, 0), (250, 0, 0), (251, 0, 0), (252, 0, 0), (253, 0, 0), (254, 0, 0)];
*/

/*
//#[allow(unused)]
pub const PALLETE_RYGB_SORTED: [(u8, u8, u8); 508] = [(0, 0, 128), (0, 0, 129), (0, 0, 130), (0, 0, 131), (0, 0, 132), (0, 0, 133), (0, 0, 134), (0, 0, 135), (0, 0, 136), (0, 0, 137), (0, 0, 138), (0, 0, 139), (0, 0, 140), (0, 0, 141), (0, 0, 142), (0, 0, 143), (0, 0, 144), (0, 0, 145), (0, 0, 146), (0, 0, 147), (0, 0, 148), (0, 0, 149), (0, 0, 150), (0, 0, 151), (0, 0, 152), (0, 0, 153), (0, 0, 154), (0, 0, 155), (0, 0, 156), (0, 0, 157), (0, 0, 158), (0, 0, 159), (0, 0, 160), (0, 0, 161), (0, 0, 162), (0, 0, 163), (0, 0, 164), (0, 0, 165), (0, 0, 166), (0, 0, 167), (0, 0, 168), (0, 0, 169), (0, 0, 170), (0, 0, 171), (0, 0, 172), (0, 0, 173), (0, 0, 174), (0, 0, 175), (0, 0, 176), (0, 0, 177), (0, 0, 178), (0, 0, 179), (0, 0, 180), (0, 0, 181), (0, 0, 182), (0, 0, 183), (0, 0, 184), (0, 0, 185), (0, 0, 186), (0, 0, 187), (0, 0, 188), (0, 0, 189), (0, 0, 190), (0, 0, 191), (0, 0, 192), (0, 0, 193), (0, 0, 194), (0, 0, 195), (0, 0, 196), (0, 0, 197), (0, 0, 198), (0, 0, 199), (0, 0, 200), (0, 0, 201), (0, 0, 202), (0, 0, 203), (0, 0, 204), (0, 0, 205), (0, 0, 206), (0, 0, 207), (0, 0, 208), (0, 0, 209), (0, 0, 210), (0, 0, 211), (0, 0, 212), (0, 0, 213), (0, 0, 214), (0, 0, 215), (0, 0, 216), (0, 0, 217), (0, 0, 218), (0, 0, 219), (0, 0, 220), (0, 0, 221), (0, 0, 222), (0, 0, 223), (0, 0, 224), (0, 0, 225), (0, 0, 226), (0, 0, 227), (0, 0, 228), (0, 0, 229), (0, 0, 230), (0, 0, 231), (0, 0, 232), (0, 0, 233), (0, 0, 234), (0, 0, 235), (0, 0, 236), (0, 0, 237), (0, 0, 238), (0, 0, 239), (0, 0, 240), (0, 0, 241), (0, 0, 242), (0, 0, 243), (0, 0, 244), (0, 0, 245), (0, 0, 246), (0, 0, 247), (0, 0, 248), (0, 0, 249), (0, 0, 250), (0, 0, 251), (0, 0, 252), (0, 0, 253), (0, 0, 254), (0, 128, 0), (0, 129, 0), (0, 130, 0), (0, 131, 0), (0, 132, 0), (0, 133, 0), (0, 134, 0), (0, 135, 0), (0, 136, 0), (0, 137, 0), (0, 138, 0), (0, 139, 0), (0, 140, 0), (0, 141, 0), (0, 142, 0), (0, 143, 0), (0, 144, 0), (0, 145, 0), (0, 146, 0), (0, 147, 0), (0, 148, 0), (0, 149, 0), (0, 150, 0), (0, 151, 0), (0, 152, 0), (0, 153, 0), (0, 154, 0), (0, 155, 0), (0, 156, 0), (0, 157, 0), (0, 158, 0), (0, 159, 0), (0, 160, 0), (0, 161, 0), (0, 162, 0), (0, 163, 0), (0, 164, 0), (0, 165, 0), (0, 166, 0), (0, 167, 0), (0, 168, 0), (0, 169, 0), (0, 170, 0), (0, 171, 0), (0, 172, 0), (0, 173, 0), (0, 174, 0), (0, 175, 0), (0, 176, 0), (0, 177, 0), (0, 178, 0), (0, 179, 0), (0, 180, 0), (0, 181, 0), (0, 182, 0), (0, 183, 0), (0, 184, 0), (0, 185, 0), (0, 186, 0), (0, 187, 0), (0, 188, 0), (0, 189, 0), (0, 190, 0), (0, 191, 0), (0, 192, 0), (0, 193, 0), (0, 194, 0), (0, 195, 0), (0, 196, 0), (0, 197, 0), (0, 198, 0), (0, 199, 0), (0, 200, 0), (0, 201, 0), (0, 202, 0), (0, 203, 0), (0, 204, 0), (0, 205, 0), (0, 206, 0), (0, 207, 0), (0, 208, 0), (0, 209, 0), (0, 210, 0), (0, 211, 0), (0, 212, 0), (0, 213, 0), (0, 214, 0), (0, 215, 0), (0, 216, 0), (0, 217, 0), (0, 218, 0), (0, 219, 0), (0, 220, 0), (0, 221, 0), (0, 222, 0), (0, 223, 0), (0, 224, 0), (0, 225, 0), (0, 226, 0), (0, 227, 0), (0, 228, 0), (0, 229, 0), (0, 230, 0), (0, 231, 0), (0, 232, 0), (0, 233, 0), (0, 234, 0), (0, 235, 0), (0, 236, 0), (0, 237, 0), (0, 238, 0), (0, 239, 0), (0, 240, 0), (0, 241, 0), (0, 242, 0), (0, 243, 0), (0, 244, 0), (0, 245, 0), (0, 246, 0), (0, 247, 0), (0, 248, 0), (0, 249, 0), (0, 250, 0), (0, 251, 0), (0, 252, 0), (0, 253, 0), (0, 254, 0), (255, 254, 0), (255, 253, 0), (255, 252, 0), (255, 251, 0), (255, 250, 0), (255, 249, 0), (255, 248, 0), (255, 247, 0), (255, 246, 0), (255, 245, 0), (255, 244, 0), (255, 243, 0), (255, 242, 0), (255, 241, 0), (255, 240, 0), (255, 239, 0), (255, 238, 0), (255, 237, 0), (255, 236, 0), (255, 235, 0), (255, 234, 0), (255, 233, 0), (255, 232, 0), (255, 231, 0), (255, 230, 0), (255, 229, 0), (255, 228, 0), (255, 227, 0), (255, 226, 0), (255, 225, 0), (255, 224, 0), (255, 223, 0), (255, 222, 0), (255, 221, 0), (255, 220, 0), (255, 219, 0), (255, 218, 0), (255, 217, 0), (255, 216, 0), (255, 215, 0), (255, 214, 0), (255, 213, 0), (255, 212, 0), (255, 211, 0), (255, 210, 0), (255, 209, 0), (255, 208, 0), (255, 207, 0), (255, 206, 0), (255, 205, 0), (255, 204, 0), (255, 203, 0), (255, 202, 0), (255, 201, 0), (255, 200, 0), (255, 199, 0), (255, 198, 0), (255, 197, 0), (255, 196, 0), (255, 195, 0), (255, 194, 0), (255, 193, 0), (255, 192, 0), (255, 191, 0), (255, 190, 0), (255, 189, 0), (255, 188, 0), (255, 187, 0), (255, 186, 0), (255, 185, 0), (255, 184, 0), (255, 183, 0), (255, 182, 0), (255, 181, 0), (255, 180, 0), (255, 179, 0), (255, 178, 0), (255, 177, 0), (255, 176, 0), (255, 175, 0), (255, 174, 0), (255, 173, 0), (255, 172, 0), (255, 171, 0), (255, 170, 0), (255, 169, 0), (255, 168, 0), (255, 167, 0), (255, 166, 0), (255, 165, 0), (255, 164, 0), (255, 163, 0), (255, 162, 0), (255, 161, 0), (255, 160, 0), (255, 159, 0), (255, 158, 0), (255, 157, 0), (255, 156, 0), (255, 155, 0), (255, 154, 0), (255, 153, 0), (255, 152, 0), (255, 151, 0), (255, 150, 0), (255, 149, 0), (255, 148, 0), (255, 147, 0), (255, 146, 0), (255, 145, 0), (255, 144, 0), (255, 143, 0), (255, 142, 0), (255, 141, 0), (255, 140, 0), (255, 139, 0), (255, 138, 0), (255, 137, 0), (255, 136, 0), (255, 135, 0), (255, 134, 0), (255, 133, 0), (255, 132, 0), (255, 131, 0), (255, 130, 0), (255, 129, 0), (255, 128, 0), (254, 0, 0), (253, 0, 0), (252, 0, 0), (251, 0, 0), (250, 0, 0), (249, 0, 0), (248, 0, 0), (247, 0, 0), (246, 0, 0), (245, 0, 0), (244, 0, 0), (243, 0, 0), (242, 0, 0), (241, 0, 0), (240, 0, 0), (239, 0, 0), (238, 0, 0), (237, 0, 0), (236, 0, 0), (235, 0, 0), (234, 0, 0), (233, 0, 0), (232, 0, 0), (231, 0, 0), (230, 0, 0), (229, 0, 0), (228, 0, 0), (227, 0, 0), (226, 0, 0), (225, 0, 0), (224, 0, 0), (223, 0, 0), (222, 0, 0), (221, 0, 0), (220, 0, 0), (219, 0, 0), (218, 0, 0), (217, 0, 0), (216, 0, 0), (215, 0, 0), (214, 0, 0), (213, 0, 0), (212, 0, 0), (211, 0, 0), (210, 0, 0), (209, 0, 0), (208, 0, 0), (207, 0, 0), (206, 0, 0), (205, 0, 0), (204, 0, 0), (203, 0, 0), (202, 0, 0), (201, 0, 0), (200, 0, 0), (199, 0, 0), (198, 0, 0), (197, 0, 0), (196, 0, 0), (195, 0, 0), (194, 0, 0), (193, 0, 0), (192, 0, 0), (191, 0, 0), (190, 0, 0), (189, 0, 0), (188, 0, 0), (187, 0, 0), (186, 0, 0), (185, 0, 0), (184, 0, 0), (183, 0, 0), (182, 0, 0), (181, 0, 0), (180, 0, 0), (179, 0, 0), (178, 0, 0), (177, 0, 0), (176, 0, 0), (175, 0, 0), (174, 0, 0), (173, 0, 0), (172, 0, 0), (171, 0, 0), (170, 0, 0), (169, 0, 0), (168, 0, 0), (167, 0, 0), (166, 0, 0), (165, 0, 0), (164, 0, 0), (163, 0, 0), (162, 0, 0), (161, 0, 0), (160, 0, 0), (159, 0, 0), (158, 0, 0), (157, 0, 0), (156, 0, 0), (155, 0, 0), (154, 0, 0), (153, 0, 0), (152, 0, 0), (151, 0, 0), (150, 0, 0), (149, 0, 0), (148, 0, 0), (147, 0, 0), (146, 0, 0), (145, 0, 0), (144, 0, 0), (143, 0, 0), (142, 0, 0), (141, 0, 0), (140, 0, 0), (139, 0, 0), (138, 0, 0), (137, 0, 0), (136, 0, 0), (135, 0, 0), (134, 0, 0), (133, 0, 0), (132, 0, 0), (131, 0, 0), (130, 0, 0), (129, 0, 0), (128, 0, 0)];
*/

/*
//#[allow(unused)]
pub const PALLETE_RYGB: [(u8, u8, u8); 508] = [(0, 0, 128), (0, 0, 129), (0, 0, 130), (0, 0, 131), (0, 0, 132), (0, 0, 133), (0, 0, 134), (0, 0, 135), (0, 0, 136), (0, 0, 137), (0, 0, 138), (0, 0, 139), (0, 0, 140), (0, 0, 141), (0, 0, 142), (0, 0, 143), (0, 0, 144), (0, 0, 145), (0, 0, 146), (0, 0, 147), (0, 0, 148), (0, 0, 149), (0, 0, 150), (0, 0, 151), (0, 0, 152), (0, 0, 153), (0, 0, 154), (0, 0, 155), (0, 0, 156), (0, 0, 157), (0, 0, 158), (0, 0, 159), (0, 0, 160), (0, 0, 161), (0, 0, 162), (0, 0, 163), (0, 0, 164), (0, 0, 165), (0, 0, 166), (0, 0, 167), (0, 0, 168), (0, 0, 169), (0, 0, 170), (0, 0, 171), (0, 0, 172), (0, 0, 173), (0, 0, 174), (0, 0, 175), (0, 0, 176), (0, 0, 177), (0, 0, 178), (0, 0, 179), (0, 0, 180), (0, 0, 181), (0, 0, 182), (0, 0, 183), (0, 0, 184), (0, 0, 185), (0, 0, 186), (0, 0, 187), (0, 0, 188), (0, 0, 189), (0, 0, 190), (0, 0, 191), (0, 0, 192), (0, 0, 193), (0, 0, 194), (0, 0, 195), (0, 0, 196), (0, 0, 197), (0, 0, 198), (0, 0, 199), (0, 0, 200), (0, 0, 201), (0, 0, 202), (0, 0, 203), (0, 0, 204), (0, 0, 205), (0, 0, 206), (0, 0, 207), (0, 0, 208), (0, 0, 209), (0, 0, 210), (0, 0, 211), (0, 0, 212), (0, 0, 213), (0, 0, 214), (0, 0, 215), (0, 0, 216), (0, 0, 217), (0, 0, 218), (0, 0, 219), (0, 0, 220), (0, 0, 221), (0, 0, 222), (0, 0, 223), (0, 0, 224), (0, 0, 225), (0, 0, 226), (0, 0, 227), (0, 0, 228), (0, 0, 229), (0, 0, 230), (0, 0, 231), (0, 0, 232), (0, 0, 233), (0, 0, 234), (0, 0, 235), (0, 0, 236), (0, 0, 237), (0, 0, 238), (0, 0, 239), (0, 0, 240), (0, 0, 241), (0, 0, 242), (0, 0, 243), (0, 0, 244), (0, 0, 245), (0, 0, 246), (0, 0, 247), (0, 0, 248), (0, 0, 249), (0, 0, 250), (0, 0, 251), (0, 0, 252), (0, 0, 253), (0, 0, 254), (0, 128, 0), (0, 129, 0), (0, 130, 0), (0, 131, 0), (0, 132, 0), (0, 133, 0), (0, 134, 0), (0, 135, 0), (0, 136, 0), (0, 137, 0), (0, 138, 0), (0, 139, 0), (0, 140, 0), (0, 141, 0), (0, 142, 0), (0, 143, 0), (0, 144, 0), (0, 145, 0), (0, 146, 0), (0, 147, 0), (0, 148, 0), (0, 149, 0), (0, 150, 0), (0, 151, 0), (0, 152, 0), (0, 153, 0), (0, 154, 0), (0, 155, 0), (0, 156, 0), (0, 157, 0), (0, 158, 0), (0, 159, 0), (0, 160, 0), (0, 161, 0), (0, 162, 0), (0, 163, 0), (0, 164, 0), (0, 165, 0), (0, 166, 0), (0, 167, 0), (0, 168, 0), (0, 169, 0), (0, 170, 0), (0, 171, 0), (0, 172, 0), (0, 173, 0), (0, 174, 0), (0, 175, 0), (0, 176, 0), (0, 177, 0), (0, 178, 0), (0, 179, 0), (0, 180, 0), (0, 181, 0), (0, 182, 0), (0, 183, 0), (0, 184, 0), (0, 185, 0), (0, 186, 0), (0, 187, 0), (0, 188, 0), (0, 189, 0), (0, 190, 0), (0, 191, 0), (0, 192, 0), (0, 193, 0), (0, 194, 0), (0, 195, 0), (0, 196, 0), (0, 197, 0), (0, 198, 0), (0, 199, 0), (0, 200, 0), (0, 201, 0), (0, 202, 0), (0, 203, 0), (0, 204, 0), (0, 205, 0), (0, 206, 0), (0, 207, 0), (0, 208, 0), (0, 209, 0), (0, 210, 0), (0, 211, 0), (0, 212, 0), (0, 213, 0), (0, 214, 0), (0, 215, 0), (0, 216, 0), (0, 217, 0), (0, 218, 0), (0, 219, 0), (0, 220, 0), (0, 221, 0), (0, 222, 0), (0, 223, 0), (0, 224, 0), (0, 225, 0), (0, 226, 0), (0, 227, 0), (0, 228, 0), (0, 229, 0), (0, 230, 0), (0, 231, 0), (0, 232, 0), (0, 233, 0), (0, 234, 0), (0, 235, 0), (0, 236, 0), (0, 237, 0), (0, 238, 0), (0, 239, 0), (0, 240, 0), (0, 241, 0), (0, 242, 0), (0, 243, 0), (0, 244, 0), (0, 245, 0), (0, 246, 0), (0, 247, 0), (0, 248, 0), (0, 249, 0), (0, 250, 0), (0, 251, 0), (0, 252, 0), (0, 253, 0), (0, 254, 0), (255, 128, 0), (255, 129, 0), (255, 130, 0), (255, 131, 0), (255, 132, 0), (255, 133, 0), (255, 134, 0), (255, 135, 0), (255, 136, 0), (255, 137, 0), (255, 138, 0), (255, 139, 0), (255, 140, 0), (255, 141, 0), (255, 142, 0), (255, 143, 0), (255, 144, 0), (255, 145, 0), (255, 146, 0), (255, 147, 0), (255, 148, 0), (255, 149, 0), (255, 150, 0), (255, 151, 0), (255, 152, 0), (255, 153, 0), (255, 154, 0), (255, 155, 0), (255, 156, 0), (255, 157, 0), (255, 158, 0), (255, 159, 0), (255, 160, 0), (255, 161, 0), (255, 162, 0), (255, 163, 0), (255, 164, 0), (255, 165, 0), (255, 166, 0), (255, 167, 0), (255, 168, 0), (255, 169, 0), (255, 170, 0), (255, 171, 0), (255, 172, 0), (255, 173, 0), (255, 174, 0), (255, 175, 0), (255, 176, 0), (255, 177, 0), (255, 178, 0), (255, 179, 0), (255, 180, 0), (255, 181, 0), (255, 182, 0), (255, 183, 0), (255, 184, 0), (255, 185, 0), (255, 186, 0), (255, 187, 0), (255, 188, 0), (255, 189, 0), (255, 190, 0), (255, 191, 0), (255, 192, 0), (255, 193, 0), (255, 194, 0), (255, 195, 0), (255, 196, 0), (255, 197, 0), (255, 198, 0), (255, 199, 0), (255, 200, 0), (255, 201, 0), (255, 202, 0), (255, 203, 0), (255, 204, 0), (255, 205, 0), (255, 206, 0), (255, 207, 0), (255, 208, 0), (255, 209, 0), (255, 210, 0), (255, 211, 0), (255, 212, 0), (255, 213, 0), (255, 214, 0), (255, 215, 0), (255, 216, 0), (255, 217, 0), (255, 218, 0), (255, 219, 0), (255, 220, 0), (255, 221, 0), (255, 222, 0), (255, 223, 0), (255, 224, 0), (255, 225, 0), (255, 226, 0), (255, 227, 0), (255, 228, 0), (255, 229, 0), (255, 230, 0), (255, 231, 0), (255, 232, 0), (255, 233, 0), (255, 234, 0), (255, 235, 0), (255, 236, 0), (255, 237, 0), (255, 238, 0), (255, 239, 0), (255, 240, 0), (255, 241, 0), (255, 242, 0), (255, 243, 0), (255, 244, 0), (255, 245, 0), (255, 246, 0), (255, 247, 0), (255, 248, 0), (255, 249, 0), (255, 250, 0), (255, 251, 0), (255, 252, 0), (255, 253, 0), (255, 254, 0), (128, 0, 0), (129, 0, 0), (130, 0, 0), (131, 0, 0), (132, 0, 0), (133, 0, 0), (134, 0, 0), (135, 0, 0), (136, 0, 0), (137, 0, 0), (138, 0, 0), (139, 0, 0), (140, 0, 0), (141, 0, 0), (142, 0, 0), (143, 0, 0), (144, 0, 0), (145, 0, 0), (146, 0, 0), (147, 0, 0), (148, 0, 0), (149, 0, 0), (150, 0, 0), (151, 0, 0), (152, 0, 0), (153, 0, 0), (154, 0, 0), (155, 0, 0), (156, 0, 0), (157, 0, 0), (158, 0, 0), (159, 0, 0), (160, 0, 0), (161, 0, 0), (162, 0, 0), (163, 0, 0), (164, 0, 0), (165, 0, 0), (166, 0, 0), (167, 0, 0), (168, 0, 0), (169, 0, 0), (170, 0, 0), (171, 0, 0), (172, 0, 0), (173, 0, 0), (174, 0, 0), (175, 0, 0), (176, 0, 0), (177, 0, 0), (178, 0, 0), (179, 0, 0), (180, 0, 0), (181, 0, 0), (182, 0, 0), (183, 0, 0), (184, 0, 0), (185, 0, 0), (186, 0, 0), (187, 0, 0), (188, 0, 0), (189, 0, 0), (190, 0, 0), (191, 0, 0), (192, 0, 0), (193, 0, 0), (194, 0, 0), (195, 0, 0), (196, 0, 0), (197, 0, 0), (198, 0, 0), (199, 0, 0), (200, 0, 0), (201, 0, 0), (202, 0, 0), (203, 0, 0), (204, 0, 0), (205, 0, 0), (206, 0, 0), (207, 0, 0), (208, 0, 0), (209, 0, 0), (210, 0, 0), (211, 0, 0), (212, 0, 0), (213, 0, 0), (214, 0, 0), (215, 0, 0), (216, 0, 0), (217, 0, 0), (218, 0, 0), (219, 0, 0), (220, 0, 0), (221, 0, 0), (222, 0, 0), (223, 0, 0), (224, 0, 0), (225, 0, 0), (226, 0, 0), (227, 0, 0), (228, 0, 0), (229, 0, 0), (230, 0, 0), (231, 0, 0), (232, 0, 0), (233, 0, 0), (234, 0, 0), (235, 0, 0), (236, 0, 0), (237, 0, 0), (238, 0, 0), (239, 0, 0), (240, 0, 0), (241, 0, 0), (242, 0, 0), (243, 0, 0), (244, 0, 0), (245, 0, 0), (246, 0, 0), (247, 0, 0), (248, 0, 0), (249, 0, 0), (250, 0, 0), (251, 0, 0), (252, 0, 0), (253, 0, 0), (254, 0, 0)];
*/

/*
//#[allow(unused)]
pub const IRON_BOW: [(u8, u8, u8); 119] = [
    (0, 0, 0), // black #000000 >>> COLDEST
    (0, 0, 36),
    (0, 0, 51),
    (0, 0, 66),
    (0, 0, 81),
    (2, 0, 90),
    (4, 0, 99),
    (7, 0, 106),
    (11, 0, 115),
    (14, 0, 119), // dark blue #0E0077
    (20, 0, 123),
    (27, 0, 128),
    (33, 0, 133),
    (41, 0, 137),
    (48, 0, 140),
    (55, 0, 143),
    (61, 0, 146),
    (66, 0, 149),
    (72, 0, 150),
    (78, 0, 151), // blue to purple #4E0097
    (84, 0, 152),
    (91, 0, 153),
    (97, 0, 155),
    (104, 0, 155),
    (110, 0, 156),
    (115, 0, 157),
    (122, 0, 157),
    (128, 0, 157),
    (134, 0, 157),
    (139, 0, 157), // dark magenta #8B009D
    (146, 0, 156),
    (152, 0, 155),
    (157, 0, 155),
    (162, 0, 155),
    (167, 0, 154),
    (171, 0, 153),
    (175, 1, 152),
    (178, 1, 151),
    (182, 2, 149),
    (185, 4, 149), //  #B90495 
    (188, 5, 147),
    (191, 6, 146),
    (193, 8, 144),
    (195, 11, 142),
    (198, 13, 139),
    (201, 17, 135),
    (203, 20, 132),
    (206, 23, 127),
    (208, 26, 121),
    (210, 29, 116), // #D21D74
    (212, 33, 111),
    (214, 37, 103),
    (217, 41, 97),
    (219, 46, 89),
    (221, 49, 78),
    (223, 53, 66),
    (224, 56, 54),
    (226, 60, 42),
    (228, 64, 30),
    (229, 68, 25), // #E54419
    (231, 72, 20),
    (232, 76, 16),
    (234, 78, 12),
    (235, 82, 10),
    (236, 86, 8),
    (237, 90, 7),
    (238, 93, 5),
    (239, 96, 4),
    (240, 100, 3),
    (241, 103, 3), // #F16703
    (241, 106, 2),
    (242, 109, 1),
    (243, 113, 1),
    (244, 116, 0),
    (244, 120, 0),
    (245, 125, 0),
    (246, 129, 0),
    (247, 133, 0),
    (248, 136, 0),
    (248, 139, 0), // #F88B00
    (249, 142, 0),
    (249, 145, 0),
    (250, 149, 0),
    (251, 154, 0),
    (252, 159, 0),
    (253, 163, 0),
    (253, 168, 0),
    (253, 172, 0),
    (254, 176, 0),
    (254, 179, 0), // #F5B300
    (254, 184, 0),
    (254, 187, 0),
    (254, 191, 0),
    (254, 195, 0),
    (254, 199, 0),
    (254, 202, 1),
    (254, 205, 2),
    (254, 208, 5),
    (254, 212, 9),
    (254, 216, 12), // yellow #F5D80C
    (255, 219, 15),
    (255, 221, 23),
    (255, 224, 32),
    (255, 227, 39),
    (255, 229, 50),
    (255, 232, 63),
    (255, 235, 75),
    (255, 238, 88),
    (255, 239, 102),
    (255, 241, 116), // light yellow #FFF174
    (255, 242, 134),
    (255, 244, 149),
    (255, 245, 164),
    (255, 247, 179),
    (255, 248, 192),
    (255, 249, 203),
    (255, 251, 216),
    (255, 253, 228),
    (255, 254, 239), // white to yellow #FFFEEF >>> HOTTEST
];
*/

//#[allow(unused)]
pub const IRON_BOW_LONG: [(u8, u8, u8); 433] = [(0, 0, 10), (0, 0, 20), (0, 0, 30), (0, 0, 37), (0, 0, 42), (0, 0, 46), (0, 0, 50), (0, 0, 54), (0, 0, 58), (0, 0, 62), (0, 0, 66), (0, 0, 70), (0, 0, 74), (0, 0, 79), (0, 0, 82), (1, 0, 85), (1, 0, 87), (2, 0, 89), (2, 0, 92), (3, 0, 94), (4, 0, 97), (4, 0, 99), (5, 0, 101), (6, 0, 103), (7, 0, 105), (8, 0, 107), (9, 0, 110), (10, 0, 112), (11, 0, 115), (12, 0, 116), (13, 0, 117), (13, 0, 118), (14, 0, 119), (16, 0, 120), (18, 0, 121), (19, 0, 123), (21, 0, 124), (23, 0, 125), (25, 0, 126), (27, 0, 128), (28, 0, 129), (30, 0, 131), (32, 0, 132), (34, 0, 133), (36, 0, 134), (38, 0, 135), (40, 0, 137), (42, 0, 137), (44, 0, 138), (46, 0, 139), (48, 0, 140), (50, 0, 141), (52, 0, 142), (54, 0, 142), (56, 0, 143), (57, 0, 144), (59, 0, 145), (60, 0, 146), (62, 0, 147), (63, 0, 147), (65, 0, 148), (66, 0, 149), (68, 0, 149), (69, 0, 150), (71, 0, 150), (73, 0, 150), (74, 0, 150), (76, 0, 151), (78, 0, 151), (79, 0, 151), (81, 0, 151), (82, 0, 152), (84, 0, 152), (86, 0, 152), (88, 0, 153), (90, 0, 153), (92, 0, 153), (93, 0, 154), (95, 0, 154), (97, 0, 155), (99, 0, 155), (100, 0, 155), (102, 0, 155), (104, 0, 155), (106, 0, 155), (108, 0, 156), (109, 0, 156), (111, 0, 156), (112, 0, 156), (113, 0, 157), (115, 0, 157), (117, 0, 157), (119, 0, 157), (120, 0, 157), (122, 0, 157), (124, 0, 157), (126, 0, 157), (127, 0, 157), (129, 0, 157), (131, 0, 157), (132, 0, 157), (134, 0, 157), (135, 0, 157), (137, 0, 157), (138, 0, 157), (139, 0, 157), (141, 0, 157), (143, 0, 156), (145, 0, 156), (147, 0, 156), (149, 0, 156), (150, 0, 155), (152, 0, 155), (153, 0, 155), (155, 0, 155), (156, 0, 155), (157, 0, 155), (159, 0, 155), (160, 0, 155), (162, 0, 155), (163, 0, 155), (164, 0, 155), (166, 0, 154), (167, 0, 154), (168, 0, 154), (169, 0, 153), (170, 0, 153), (171, 0, 153), (173, 0, 153), (174, 1, 152), (175, 1, 152), (176, 1, 152), (176, 1, 152), (177, 1, 151), (178, 1, 151), (179, 1, 150), (180, 2, 150), (181, 2, 149), (182, 2, 149), (183, 3, 149), (184, 3, 149), (185, 4, 149), (186, 4, 149), (186, 4, 148), (187, 5, 147), (188, 5, 147), (189, 5, 147), (190, 6, 146), (191, 6, 146), (191, 6, 146), (192, 7, 145), (192, 7, 145), (193, 8, 144), (193, 9, 144), (194, 10, 143), (195, 10, 142), (195, 11, 142), (196, 12, 141), (197, 12, 140), (198, 13, 139), (198, 14, 138), (199, 15, 137), (200, 16, 136), (201, 17, 135), (202, 18, 134), (202, 19, 133), (203, 19, 133), (203, 20, 132), (204, 21, 130), (205, 22, 129), (206, 23, 128), (206, 24, 126), (207, 24, 124), (207, 25, 123), (208, 26, 121), (209, 27, 120), (209, 28, 118), (210, 28, 117), (210, 29, 116), (211, 30, 114), (211, 32, 113), (212, 33, 111), (212, 34, 110), (213, 35, 107), (213, 36, 105), (214, 37, 103), (215, 38, 101), (216, 39, 100), (216, 40, 98), (217, 42, 96), (218, 43, 94), (218, 44, 92), (219, 46, 90), (219, 47, 87), (220, 47, 84), (221, 48, 81), (221, 49, 78), (222, 50, 74), (222, 51, 71), (223, 52, 68), (223, 53, 65), (223, 54, 61), (224, 55, 58), (224, 56, 55), (224, 57, 51), (225, 58, 48), (226, 59, 45), (226, 60, 42), (227, 61, 38), (227, 62, 35), (228, 63, 32), (228, 65, 29), (228, 66, 28), (229, 67, 27), (229, 68, 25), (229, 69, 24), (230, 70, 22), (231, 71, 21), (231, 72, 20), (231, 73, 19), (232, 74, 18), (232, 76, 16), (232, 76, 15), (233, 77, 14), (233, 77, 13), (234, 78, 12), (234, 79, 12), (235, 80, 11), (235, 81, 10), (235, 82, 10), (235, 83, 9), (236, 84, 9), (236, 86, 8), (236, 87, 8), (236, 88, 8), (237, 89, 7), (237, 90, 7), (237, 91, 6), (238, 92, 6), (238, 92, 5), (238, 93, 5), (238, 94, 5), (239, 95, 4), (239, 96, 4), (239, 97, 4), (239, 98, 4), (240, 99, 3), (240, 100, 3), (240, 101, 3), (241, 102, 3), (241, 102, 3), (241, 103, 3), (241, 104, 3), (241, 105, 2), (241, 106, 2), (241, 107, 2), (241, 107, 2), (242, 108, 1), (242, 109, 1), (242, 110, 1), (243, 111, 1), (243, 112, 1), (243, 113, 1), (243, 114, 1), (244, 115, 0), (244, 116, 0), (244, 117, 0), (244, 118, 0), (244, 119, 0), (244, 120, 0), (244, 122, 0), (245, 123, 0), (245, 124, 0), (245, 126, 0), (245, 127, 0), (246, 128, 0), (246, 129, 0), (246, 130, 0), (247, 131, 0), (247, 132, 0), (247, 133, 0), (247, 134, 0), (248, 135, 0), (248, 136, 0), (248, 136, 0), (248, 137, 0), (248, 138, 0), (248, 139, 0), (248, 140, 0), (249, 141, 0), (249, 141, 0), (249, 142, 0), (249, 143, 0), (249, 144, 0), (249, 145, 0), (249, 146, 0), (249, 147, 0), (250, 148, 0), (250, 149, 0), (250, 150, 0), (251, 152, 0), (251, 153, 0), (251, 154, 0), (251, 156, 0), (252, 157, 0), (252, 159, 0), (252, 160, 0), (252, 161, 0), (253, 162, 0), (253, 163, 0), (253, 164, 0), (253, 166, 0), (253, 167, 0), (253, 168, 0), (253, 170, 0), (253, 171, 0), (253, 172, 0), (253, 173, 0), (253, 174, 0), (254, 175, 0), (254, 176, 0), (254, 177, 0), (254, 178, 0), (254, 179, 0), (254, 180, 0), (254, 181, 0), (254, 182, 0), (254, 184, 0), (254, 185, 0), (254, 185, 0), (254, 186, 0), (254, 187, 0), (254, 188, 0), (254, 189, 0), (254, 190, 0), (254, 192, 0), (254, 193, 0), (254, 194, 0), (254, 195, 0), (254, 196, 0), (254, 197, 0), (254, 198, 0), (254, 199, 0), (254, 200, 0), (254, 201, 1), (254, 202, 1), (254, 202, 1), (254, 203, 1), (254, 204, 2), (254, 205, 2), (254, 206, 3), (254, 207, 4), (254, 207, 4), (254, 208, 5), (254, 209, 6), (254, 211, 8), (254, 212, 9), (254, 213, 10), (254, 214, 10), (254, 215, 11), (254, 216, 12), (254, 217, 13), (255, 218, 14), (255, 218, 14), (255, 219, 16), (255, 220, 18), (255, 220, 20), (255, 221, 22), (255, 222, 25), (255, 222, 27), (255, 223, 30), (255, 224, 32), (255, 225, 34), (255, 226, 36), (255, 226, 38), (255, 227, 40), (255, 228, 43), (255, 228, 46), (255, 229, 49), (255, 230, 53), (255, 230, 56), (255, 231, 60), (255, 232, 63), (255, 233, 67), (255, 234, 70), (255, 235, 73), (255, 235, 77), (255, 236, 80), (255, 237, 84), (255, 238, 87), (255, 238, 91), (255, 238, 95), (255, 239, 99), (255, 239, 103), (255, 240, 106), (255, 240, 110), (255, 241, 114), (255, 241, 119), (255, 241, 123), (255, 242, 128), (255, 242, 133), (255, 242, 138), (255, 243, 142), (255, 244, 146), (255, 244, 150), (255, 244, 154), (255, 245, 158), (255, 245, 162), (255, 245, 166), (255, 246, 170), (255, 246, 175), (255, 247, 179), (255, 247, 182), (255, 248, 186), (255, 248, 189), (255, 248, 193), (255, 248, 196), (255, 249, 199), (255, 249, 202), (255, 249, 205), (255, 250, 209), (255, 250, 212), (255, 251, 216), (255, 252, 219), (255, 252, 223), (255, 253, 226), (255, 253, 229), (255, 253, 232), (255, 254, 235), (255, 254, 238), (255, 254, 241), (255, 254, 244), (255, 255, 246)];

/*
//#[allow(unused)]
pub const IRON_BOW_LONG_ROW: [(u8, u8, u8); 433] = [
(0, 0, 10),
(0, 0, 20),
(0, 0, 30),
(0, 0, 37),
(0, 0, 42),
(0, 0, 46),
(0, 0, 50),
(0, 0, 54),
(0, 0, 58),
(0, 0, 62),
(0, 0, 66),
(0, 0, 70),
(0, 0, 74),
(0, 0, 79),
(0, 0, 82),
(1, 0, 85),
(1, 0, 87),
(2, 0, 89),
(2, 0, 92),
(3, 0, 94),
(4, 0, 97),
(4, 0, 99),
(5, 0, 101),
(6, 0, 103),
(7, 0, 105),
(8, 0, 107),
(9, 0, 110),
(10, 0, 112),
(11, 0, 115),
(12, 0, 116),
(13, 0, 117),
(13, 0, 118),
(14, 0, 119),
(16, 0, 120),
(18, 0, 121),
(19, 0, 123),
(21, 0, 124),
(23, 0, 125),
(25, 0, 126),
(27, 0, 128),
(28, 0, 129),
(30, 0, 131),
(32, 0, 132),
(34, 0, 133),
(36, 0, 134),
(38, 0, 135),
(40, 0, 137),
(42, 0, 137),
(44, 0, 138),
(46, 0, 139),
(48, 0, 140),
(50, 0, 141),
(52, 0, 142),
(54, 0, 142),
(56, 0, 143),
(57, 0, 144),
(59, 0, 145),
(60, 0, 146),
(62, 0, 147),
(63, 0, 147),
(65, 0, 148),
(66, 0, 149),
(68, 0, 149),
(69, 0, 150),
(71, 0, 150),
(73, 0, 150),
(74, 0, 150),
(76, 0, 151),
(78, 0, 151),
(79, 0, 151),
(81, 0, 151),
(82, 0, 152),
(84, 0, 152),
(86, 0, 152),
(88, 0, 153),
(90, 0, 153),
(92, 0, 153),
(93, 0, 154),
(95, 0, 154),
(97, 0, 155),
(99, 0, 155),
(100, 0, 155),
(102, 0, 155),
(104, 0, 155),
(106, 0, 155),
(108, 0, 156),
(109, 0, 156),
(111, 0, 156),
(112, 0, 156),
(113, 0, 157),
(115, 0, 157),
(117, 0, 157),
(119, 0, 157),
(120, 0, 157),
(122, 0, 157),
(124, 0, 157),
(126, 0, 157),
(127, 0, 157),
(129, 0, 157),
(131, 0, 157),
(132, 0, 157),
(134, 0, 157),
(135, 0, 157),
(137, 0, 157),
(138, 0, 157),
(139, 0, 157),
(141, 0, 157),
(143, 0, 156),
(145, 0, 156),
(147, 0, 156),
(149, 0, 156),
(150, 0, 155),
(152, 0, 155),
(153, 0, 155),
(155, 0, 155),
(156, 0, 155),
(157, 0, 155),
(159, 0, 155),
(160, 0, 155),
(162, 0, 155),
(163, 0, 155),
(164, 0, 155),
(166, 0, 154),
(167, 0, 154),
(168, 0, 154),
(169, 0, 153),
(170, 0, 153),
(171, 0, 153),
(173, 0, 153),
(174, 1, 152),
(175, 1, 152),
(176, 1, 152),
(176, 1, 152),
(177, 1, 151),
(178, 1, 151),
(179, 1, 150),
(180, 2, 150),
(181, 2, 149),
(182, 2, 149),
(183, 3, 149),
(184, 3, 149),
(185, 4, 149),
(186, 4, 149),
(186, 4, 148),
(187, 5, 147),
(188, 5, 147),
(189, 5, 147),
(190, 6, 146),
(191, 6, 146),
(191, 6, 146),
(192, 7, 145),
(192, 7, 145),
(193, 8, 144),
(193, 9, 144),
(194, 10, 143),
(195, 10, 142),
(195, 11, 142),
(196, 12, 141),
(197, 12, 140),
(198, 13, 139),
(198, 14, 138),
(199, 15, 137),
(200, 16, 136),
(201, 17, 135),
(202, 18, 134),
(202, 19, 133),
(203, 19, 133),
(203, 20, 132),
(204, 21, 130),
(205, 22, 129),
(206, 23, 128),
(206, 24, 126),
(207, 24, 124),
(207, 25, 123),
(208, 26, 121),
(209, 27, 120),
(209, 28, 118),
(210, 28, 117),
(210, 29, 116),
(211, 30, 114),
(211, 32, 113),
(212, 33, 111),
(212, 34, 110),
(213, 35, 107),
(213, 36, 105),
(214, 37, 103),
(215, 38, 101),
(216, 39, 100),
(216, 40, 98),
(217, 42, 96),
(218, 43, 94),
(218, 44, 92),
(219, 46, 90),
(219, 47, 87),
(220, 47, 84),
(221, 48, 81),
(221, 49, 78),
(222, 50, 74),
(222, 51, 71),
(223, 52, 68),
(223, 53, 65),
(223, 54, 61),
(224, 55, 58),
(224, 56, 55),
(224, 57, 51),
(225, 58, 48),
(226, 59, 45),
(226, 60, 42),
(227, 61, 38),
(227, 62, 35),
(228, 63, 32),
(228, 65, 29),
(228, 66, 28),
(229, 67, 27),
(229, 68, 25),
(229, 69, 24),
(230, 70, 22),
(231, 71, 21),
(231, 72, 20),
(231, 73, 19),
(232, 74, 18),
(232, 76, 16),
(232, 76, 15),
(233, 77, 14),
(233, 77, 13),
(234, 78, 12),
(234, 79, 12),
(235, 80, 11),
(235, 81, 10),
(235, 82, 10),
(235, 83, 9),
(236, 84, 9),
(236, 86, 8),
(236, 87, 8),
(236, 88, 8),
(237, 89, 7),
(237, 90, 7),
(237, 91, 6),
(238, 92, 6),
(238, 92, 5),
(238, 93, 5),
(238, 94, 5),
(239, 95, 4),
(239, 96, 4),
(239, 97, 4),
(239, 98, 4),
(240, 99, 3),
(240, 100, 3),
(240, 101, 3),
(241, 102, 3),
(241, 102, 3),
(241, 103, 3),
(241, 104, 3),
(241, 105, 2),
(241, 106, 2),
(241, 107, 2),
(241, 107, 2),
(242, 108, 1),
(242, 109, 1),
(242, 110, 1),
(243, 111, 1),
(243, 112, 1),
(243, 113, 1),
(243, 114, 1),
(244, 115, 0),
(244, 116, 0),
(244, 117, 0),
(244, 118, 0),
(244, 119, 0),
(244, 120, 0),
(244, 122, 0),
(245, 123, 0),
(245, 124, 0),
(245, 126, 0),
(245, 127, 0),
(246, 128, 0),
(246, 129, 0),
(246, 130, 0),
(247, 131, 0),
(247, 132, 0),
(247, 133, 0),
(247, 134, 0),
(248, 135, 0),
(248, 136, 0),
(248, 136, 0),
(248, 137, 0),
(248, 138, 0),
(248, 139, 0),
(248, 140, 0),
(249, 141, 0),
(249, 141, 0),
(249, 142, 0),
(249, 143, 0),
(249, 144, 0),
(249, 145, 0),
(249, 146, 0),
(249, 147, 0),
(250, 148, 0),
(250, 149, 0),
(250, 150, 0),
(251, 152, 0),
(251, 153, 0),
(251, 154, 0),
(251, 156, 0),
(252, 157, 0),
(252, 159, 0),
(252, 160, 0),
(252, 161, 0),
(253, 162, 0),
(253, 163, 0),
(253, 164, 0),
(253, 166, 0),
(253, 167, 0),
(253, 168, 0),
(253, 170, 0),
(253, 171, 0),
(253, 172, 0),
(253, 173, 0),
(253, 174, 0),
(254, 175, 0),
(254, 176, 0),
(254, 177, 0),
(254, 178, 0),
(254, 179, 0),
(254, 180, 0),
(254, 181, 0),
(254, 182, 0),
(254, 184, 0),
(254, 185, 0),
(254, 185, 0),
(254, 186, 0),
(254, 187, 0),
(254, 188, 0),
(254, 189, 0),
(254, 190, 0),
(254, 192, 0),
(254, 193, 0),
(254, 194, 0),
(254, 195, 0),
(254, 196, 0),
(254, 197, 0),
(254, 198, 0),
(254, 199, 0),
(254, 200, 0),
(254, 201, 1),
(254, 202, 1),
(254, 202, 1),
(254, 203, 1),
(254, 204, 2),
(254, 205, 2),
(254, 206, 3),
(254, 207, 4),
(254, 207, 4),
(254, 208, 5),
(254, 209, 6),
(254, 211, 8),
(254, 212, 9),
(254, 213, 10),
(254, 214, 10),
(254, 215, 11),
(254, 216, 12),
(254, 217, 13),
(255, 218, 14),
(255, 218, 14),
(255, 219, 16),
(255, 220, 18),
(255, 220, 20),
(255, 221, 22),
(255, 222, 25),
(255, 222, 27),
(255, 223, 30),
(255, 224, 32),
(255, 225, 34),
(255, 226, 36),
(255, 226, 38),
(255, 227, 40),
(255, 228, 43),
(255, 228, 46),
(255, 229, 49),
(255, 230, 53),
(255, 230, 56),
(255, 231, 60),
(255, 232, 63),
(255, 233, 67),
(255, 234, 70),
(255, 235, 73),
(255, 235, 77),
(255, 236, 80),
(255, 237, 84),
(255, 238, 87),
(255, 238, 91),
(255, 238, 95),
(255, 239, 99),
(255, 239, 103),
(255, 240, 106),
(255, 240, 110),
(255, 241, 114),
(255, 241, 119),
(255, 241, 123),
(255, 242, 128),
(255, 242, 133),
(255, 242, 138),
(255, 243, 142),
(255, 244, 146),
(255, 244, 150),
(255, 244, 154),
(255, 245, 158),
(255, 245, 162),
(255, 245, 166),
(255, 246, 170),
(255, 246, 175),
(255, 247, 179),
(255, 247, 182),
(255, 248, 186),
(255, 248, 189),
(255, 248, 193),
(255, 248, 196),
(255, 249, 199),
(255, 249, 202),
(255, 249, 205),
(255, 250, 209),
(255, 250, 212),
(255, 251, 216),
(255, 252, 219),
(255, 252, 223),
(255, 253, 226),
(255, 253, 229),
(255, 253, 232),
(255, 254, 235),
(255, 254, 238),
(255, 254, 241),
(255, 254, 244),
(255, 255, 246),
];
*/

//https://www.rapidtables.com/web/color/RGB_Color.html
//https://stackoverflow.com/questions/28495390/thermal-imaging-palette
//https://jsfiddle.net/gjruwftd/2/
