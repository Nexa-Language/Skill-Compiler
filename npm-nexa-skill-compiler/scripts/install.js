#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('\n📦 Nexa Skill Compiler - Post Install\n');
console.log('Checking for nexa-skill binary...');

// Check if binary exists in PATH
function binaryExists() {
  try {
    execSync('nexa-skill --version', { stdio: 'ignore' });
    return true;
  } catch (e) {
    return false;
  }
}

if (binaryExists()) {
  console.log('✅ nexa-skill binary found in PATH.\n');
  process.exit(0);
}

console.log('⚠️  nexa-skill binary not found in PATH.');
console.log('\nTo complete installation, please run one of the following:\n');
console.log('  \x1b[36m# Option 1: Install via cargo (recommended)\x1b[0m');
console.log('  cargo install nexa-skill-cli\n');
console.log('  \x1b[36m# Option 2: Build from source\x1b[0m');
console.log('  git clone https://github.com/ouyangyipeng/Skill-Compiler');
console.log('  cd Skill-Compiler');
console.log('  cargo install --path nexa-skill-cli\n');
console.log('📚 Documentation: https://github.com/ouyangyipeng/Skill-Compiler\n');