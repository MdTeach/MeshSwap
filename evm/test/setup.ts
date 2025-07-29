/**
 * Jest setup file for MeshSwap tests
 * This file runs before each test suite
 */

// Extend Jest matchers if needed
// import 'jest-extended';

// Set up test environment variables
process.env.NODE_ENV = 'test';

// Global test configuration
beforeAll(async () => {
  // Any global setup before all tests
  console.log('🧪 Setting up Jest test environment...');
});

afterAll(async () => {
  // Any global cleanup after all tests
  console.log('🧹 Cleaning up Jest test environment...');
});

// Global error handler for unhandled promises
process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection at:', promise, 'reason:', reason);
});
