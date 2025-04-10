use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub fn write_ppm_file(buf: &[[f32; 4]], width: u32, height: u32) -> std::io::Result<()> {
    let file = File::create("out.ppm")?;
    let mut writer = BufWriter::new(file);

    write!(&mut writer, "P3\n{width} {height}\n255\n")?;

    for v in buf {
        let ir = (v[0].sqrt().clamp(0.0, 0.999) * 256.0) as i32;
        let ig = (v[1].sqrt().clamp(0.0, 0.999) * 256.0) as i32;
        let ib = (v[2].sqrt().clamp(0.0, 0.999) * 256.0) as i32;

        writeln!(&mut writer, "{ir} {ig} {ib}").unwrap();
    }

    Ok(())
}
