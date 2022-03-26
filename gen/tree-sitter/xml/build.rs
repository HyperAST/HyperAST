use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["tree-sitter-xml", "src"].iter().collect();

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .compile("tree-sitter-xml");
}
//(source_file (prolog (XMLDecl (VersionInfo (VersionNum)) (EncodingDecl (EncName))) (ERROR) (Comment)) (element (STag (Name) (Attribute (Name) (AttValue)) (Attribute (Name) (ERROR) (AttValue)) (Attribute (Name) (ERROR) (AttValue)))
// <?xml version="1.0" encoding="UTF-8"?>
// <!--
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License. See accompanying LICENSE file.
// -->
// <project xmlns="http://maven.apache.org/POM/4.0.0"
// xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
// xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 https://maven.apache.org/xsd/maven-4.0.0.xsd">
//   <modelVersion>4.0.0</modelVersion>