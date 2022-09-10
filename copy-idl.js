const fs = require('fs');
const idl = require('./target/idl/multisig_wallet.json');

fs.writeFileSync('./app/src/idl.json', JSON.stringify(idl));