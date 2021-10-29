#!/usr/bin/env node

import fs from 'fs';
import path from 'path';

const OUTPUT = 'npm_dist/';
const HEADER = `// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
`;

function copyFile (file) {
	writeFile(file, fs.readFileSync(file, 'utf-8'));
}

function writeFile (file, contents) {
	fs.writeFileSync(path.join(OUTPUT, file), contents);
}

function writeWithHeader (file, contents) {
	writeFile(file, `${HEADER}\n${contents}`);
}

function main () {
	const typesD = fs.readFileSync('types.d.ts', 'utf-8');
	const pkgJson = JSON.parse(fs.readFileSync('package.json', 'utf-8'));
	const { registry } = JSON.parse(fs.readFileSync('ss58-registry.json', 'utf-8'));

	// mangle the code output into something JS-like
	const code = JSON.stringify(registry, null, 2)
		.replace(/\n    "/g, '\n\t\t') // change the leading key " into '
		.replace(/":/g, ':') // change the trailing key ": into :
		.replace(/"/g, "'") // use single quotes elsewhere
		.replace(/  /g, '\t'); // change all spaces into tabs

	pkgJson.exports = {
		'.': {
			require: './index.cjs',
			default: './index.js'
		},
	};
	pkgJson.main = 'index.js';
	pkgJson.type = 'module';

	delete pkgJson.scripts;

	writeWithHeader('index.cjs', `module.exports = ${code};\n`);
	writeWithHeader('index.js', `export default ${code};\n`);

	writeFile('package.json', JSON.stringify(pkgJson, null, 2));
	writeFile('index.d.ts', `${typesD}\ndeclare const _default: Registry;\n\nexport default _default;\n`);

	copyFile('CHANGELOG.md');
	copyFile('README.md');
	copyFile('LICENSE');
}

main();
