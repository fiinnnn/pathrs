use std::{
    fs::File,
    io::{BufWriter, Write},
};

use bevy::math::Vec3;

// leaving this in for debug purposes
#[allow(unused)]
pub fn write_ppm_file(buf: &[Vec3], width: usize, height: usize) -> std::io::Result<()> {
    let file = File::create("out.ppm")?;
    let mut writer = BufWriter::new(file);

    write!(&mut writer, "P3\n{width} {height}\n255\n")?;

    for v in buf {
        let ir = (v.x.clamp(0.0, 0.999) * 256.0) as i32;
        let ig = (v.y.clamp(0.0, 0.999) * 256.0) as i32;
        let ib = (v.z.clamp(0.0, 0.999) * 256.0) as i32;

        writeln!(&mut writer, "{ir} {ig} {ib}").unwrap();
    }

    Ok(())
}
