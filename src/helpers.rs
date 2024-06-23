// This file is part of pycavalry.
// 
// pycavalry is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use ruff_text_size::TextRange;

pub fn read_exact_from_file(file: &str, range: TextRange) -> String {
    return file[range.start().to_usize()..range.end().to_usize()].to_owned();
}

pub fn read_exact_from_file_name(file_name: &PathBuf, range: TextRange) -> String {
    let mut buf = vec![0; (range.end() - range.start()).to_usize()];
    let mut file = File::open(file_name).unwrap();
    file.seek(SeekFrom::Start(range.start().to_u32() as u64))
        .unwrap();
    file.read_exact(&mut buf).expect("Error reading file");
    String::from_utf8(buf).unwrap()
}
