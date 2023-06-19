use std::{fs::OpenOptions, io::BufWriter, path::Path};

use ptrsx::{s64::Params, sc64::PtrsxScanner};
use rayon::prelude::*;

use super::{
    cmd::SubCommandScan,
    utils::{select_module, Spinner},
};

impl SubCommandScan {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandScan { ref file, target, depth, ignore, offset, dir } = self;
        let mut ptrsx = PtrsxScanner::default();
        let file_name = file.file_stem().and_then(|f| f.to_str()).ok_or("get filename error")?;
        let mut spinner = Spinner::start("Start loading cache...");
        ptrsx.load_pointer_map_file(file)?;
        spinner.stop("cache loaded.");

        let pages = select_module(ptrsx.pages())?;
        let mut spinner = Spinner::start("Start creating pointer maps...");
        let rev_map = ptrsx.get_rev_pointer_map();
        spinner.stop("Pointer map is created.");

        let dir = dir.unwrap_or_default();

        let mut spinner = Spinner::start("Start scanning pointer chain...");
        pages
            .par_iter()
            .map(|m| (m.start, m.path, ptrsx.range_address(m).collect::<Vec<_>>()))
            .try_for_each(|(base, name, points)| {
                let name = Path::new(name)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .expect("get region name error");
                let file = dir.join(format!("{file_name}-{name}.scandata"));
                let file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create_new(true)
                    .open(file)?;
                let params = Params {
                    base: base as usize,
                    depth,
                    ignore,
                    range: offset.0,
                    points: &points,
                    target: target.0,
                    writer: &mut BufWriter::new(file),
                };
                ptrsx.scan(&rev_map, params)
            })?;
        spinner.stop("Pointer chain is scanned.");

        Ok(())
    }
}
