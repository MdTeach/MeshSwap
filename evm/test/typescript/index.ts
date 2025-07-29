import { ethers } from 'ethers';
import { MeshSwap } from '../../src/index';

/**
 * TypeScript tests for MeshSwap contracts
 * These tests complement the Foundry Solidity tests
 */

async function testConnection() {
  console.log('Testing connection to local node...');
  
  const meshSwap = new MeshSwap('http://localhost:8545');
  
  try {
    const blockNumber = await meshSwap.getBlockNumber();
    console.log(`✅ Connected! Current block: ${blockNumber}`);
    return true;
  } catch (error) {
    console.log(`❌ Connection failed: ${error}`);
    return false;
  }
}

async function testBalance() {
  console.log('Testing balance retrieval...');
  
  const meshSwap = new MeshSwap('http://localhost:8545');
  
  try {
    // Test with a known address (first Anvil account)
    const testAddress = '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266';
    const balance = await meshSwap.getBalance(testAddress);
    console.log(`✅ Balance for ${testAddress}: ${balance} ETH`);
    return true;
  } catch (error) {
    console.log(`❌ Balance test failed: ${error}`);
    return false;
  }
}

async function runTests() {
  console.log('🧪 Running TypeScript tests for MeshSwap...\n');
  
  const tests = [
    { name: 'Connection Test', fn: testConnection },
    { name: 'Balance Test', fn: testBalance },
  ];
  
  let passed = 0;
  let failed = 0;
  
  for (const test of tests) {
    console.log(`Running ${test.name}...`);
    const result = await test.fn();
    
    if (result) {
      passed++;
    } else {
      failed++;
    }
    
    console.log('');
  }
  
  console.log(`📊 Test Results: ${passed} passed, ${failed} failed`);
  
  if (failed > 0) {
    process.exit(1);
  }
}

if (require.main === module) {
  runTests().catch(console.error);
}
