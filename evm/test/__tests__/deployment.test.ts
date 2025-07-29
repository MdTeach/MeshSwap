import { ContractDeployer } from '../../scripts/deploy';

describe('ContractDeployer', () => {
  const testRpcUrl = 'http://localhost:8545';
  const testPrivateKey = '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80';

  let deployer: ContractDeployer;

  beforeEach(() => {
    deployer = new ContractDeployer(testRpcUrl, testPrivateKey);
  });

  describe('Constructor', () => {
    it('should create ContractDeployer instance', () => {
      expect(deployer).toBeInstanceOf(ContractDeployer);
    });

    it('should throw error with invalid private key', () => {
      expect(() => {
        new ContractDeployer(testRpcUrl, 'invalid-key');
      }).toThrow();
    });
  });

  describe('deployContract', () => {
    it('should throw error for non-existent contract artifact', async () => {
      await expect(
        deployer.deployContract('NonExistentContract')
      ).rejects.toThrow('Contract artifact not found');
    });
  });

  describe('verifyDeployment', () => {
    it('should handle verification of mock contract', async () => {
      // Mock contract object for testing
      const mockContract = {
        name: jest.fn().mockResolvedValue('TestContract'),
      } as any;

      // This should not throw
      await expect(
        deployer.verifyDeployment(mockContract, 'name')
      ).resolves.toBeUndefined();

      expect(mockContract.name).toHaveBeenCalled();
    });

    it('should handle verification failure gracefully', async () => {
      const mockContract = {
        name: jest.fn().mockRejectedValue(new Error('Function not found')),
      } as any;

      // Should not throw, but log error
      await expect(
        deployer.verifyDeployment(mockContract, 'name')
      ).resolves.toBeUndefined();

      expect(mockContract.name).toHaveBeenCalled();
    });
  });
});
