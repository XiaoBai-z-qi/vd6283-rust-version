use super::types::{AlsData, LuxCct};

const XYZ_MATRIX:                       [[f32; 3]; 3] = [
    [0.205570, 0.416700, -0.143816],
    [-0.028752, 0.506372, -0.120614],
    [-0.552625, 0.335866, 0.494781],
];

pub const LUX_REFERENCE_EXPOSURE_US:    u32 = 100_800;

pub fn normalize_als(measurement: u32, exposure_us: u32, gain: u16) -> f32 {
    if exposure_us == 0 || gain == 0 {
        return 0.0;
    }
    let exposure_scale = LUX_REFERENCE_EXPOSURE_US as f32 / exposure_us as f32;
    let measurement = measurement as f32 / 256.0;
    let gain = gain as f32 / 256.0;
    exposure_scale * measurement / gain
}

pub fn get_lux_cct(als: &AlsData, exposure_us: u32) -> LuxCct {
    let red = normalize_als(als.count_value[0], exposure_us, als.gains[0]);
    let green = normalize_als(als.count_value[3], exposure_us, als.gains[3]);
    let blue = normalize_als(als.count_value[2], exposure_us, als.gains[2]);
    let rgb = [red, green, blue];

    let x = dot(XYZ_MATRIX[0], rgb);
    let y = dot(XYZ_MATRIX[1], rgb);
    let z = dot(XYZ_MATRIX[2], rgb);
    let lux = y.max(0.0);

    let cct = {
        let sum = x + y + z;
        if sum == 0.0 {
            0.0
        } else {
            let chromaticity_x = x / sum;
            let chromaticity_y = y / sum;
            let denominator = 0.1858 - chromaticity_y;
            if denominator == 0.0 {
                0.0
            } else {
                let n = (chromaticity_x - 0.3320) / denominator;
                let n2 = n * n;
                let n3 = n2 * n;
                449.0 * n3 + 3525.0 * n2 + 6823.3 * n + 5520.33
            }
        }
    };

    LuxCct { x, y, z, lux, cct }
}

fn dot(row: [f32; 3], rgb: [f32; 3]) -> f32 {
    row[0] * rgb[0] + row[1] * rgb[1] + row[2] * rgb[2]
}
