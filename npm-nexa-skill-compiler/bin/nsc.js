#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Binary name based on platform
function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;
  
  if (platform === 'win32') {
    return 'nexa-skill.exe';
  }
  return 'nexa-skill';
}

// Find binary location
function findBinary() {
  // Check if installed via cargo
  const cargoBin = path.join(process.env.CARGO_HOME || path.join(process.env.HOME || process.env.USERPROFILE, '.cargo', 'bin'), getBinaryName());
  
  // Check local node_modules
  const localBin = path.join(__dirname, '..', 'binaries', process.platform, process.arch, getBinaryName());
  
  // Check PATH
  const pathBinary = getBinaryName();
  
  if (fs.existsSync(cargoBin)) {
    return cargoBin;
  }
  if (fs.existsSync(localBin)) {
    return localBin;
  }
  
  // Return the binary name and let the system find it in PATH
  return pathBinary;
}

// Run the binary
const binary = findBinary();
const args = process.argv.slice(2);

const child = spawn(binary, args, {
  stdio: 'inherit',
  shell: process.platform === 'win32'
});

child.on('error', (err) => {
  if (err.code === 'ENOENT') {
    console.error('\n\x1b[31mError: nexa-skill binary not found.\x1b[0m');
    console.error('\nPlease install the Rust binary first:');
    console.error('  \x1b[36mcargo install nexa-skill-cli\x1b[0m');
    console.error('\nOr visit: https://github.com/ouyangyipeng/Skill-Compiler\n');
    process.exit(1);
  } else {
    console.error('Error running nexa-skill:', err.message);
    process.exit(1);
  }
});

child.on('exit', (code) => {
  process.exit(code || 0);
});