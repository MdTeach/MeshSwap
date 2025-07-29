import { ethers } from 'ethers';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Deployment script for MeshSwap contracts
 * This script works with Foundry-compiled artifacts
 */

interface ContractArtifact {
  abi: any[];
  bytecode: {
    object: string;
  };
}

export class ContractDeployer {
  private provider: ethers.Provider;
  private signer: ethers.Signer;

  constructor(providerUrl: string, privateKey: string) {
    this.provider = new ethers.JsonRpcProvider(providerUrl);
    this.signer = new ethers.Wallet(privateKey, this.provider);
  }

  /**
   * Load contract artifact from Foundry's out directory
   */
  private loadArtifact(contractName: string): ContractArtifact {
    const artifactPath = path.join(__dirname, '..', 'out', `${contractName}.sol`, `${contractName}.json`);
    
    if (!fs.existsSync(artifactPath)) {
      throw new Error(`Contract artifact not found: ${artifactPath}`);
    }

    return JSON.parse(fs.readFileSync(artifactPath, 'utf8'));
  }

  /**
   * Deploy a contract
   */
  async deployContract(
    contractName: string,
    constructorArgs: any[] = [],
    options: { gasLimit?: number; gasPrice?: bigint } = {}
  ): Promise<ethers.BaseContract> {
    console.log(`Deploying ${contractName}...`);

    const artifact = this.loadArtifact(contractName);
    const factory = new ethers.ContractFactory(
      artifact.abi,
      artifact.bytecode.object,
      this.signer
    );

    const deployTx = await factory.deploy(...constructorArgs, {
      gasLimit: options.gasLimit,
      gasPrice: options.gasPrice,
    });

    console.log(`Transaction hash: ${deployTx.deploymentTransaction()?.hash}`);
    
    const contract = await deployTx.waitForDeployment();
    const address = await contract.getAddress();
    
    console.log(`${contractName} deployed to: ${address}`);
    
    return contract;
  }

  /**
   * Verify deployment by calling a view function
   */
  async verifyDeployment(contract: ethers.BaseContract, functionName: string = 'name'): Promise<void> {
    try {
      // Type assertion to access contract functions dynamically
      const contractWithMethods = contract as any;
      const result = await contractWithMethods[functionName]();
      console.log(`Verification successful: ${functionName}() = ${result}`);
    } catch (error) {
      console.error('Verification failed:', error);
    }
  }
}

// Example deployment script
async function main() {
  const providerUrl = process.env.RPC_URL || 'http://localhost:8545';
  const privateKey = process.env.PRIVATE_KEY;

  if (!privateKey) {
    throw new Error('PRIVATE_KEY environment variable is required');
  }

  const deployer = new ContractDeployer(providerUrl, privateKey);

  try {
    // Example: Deploy your main contract
    // const meshSwapContract = await deployer.deployContract('MeshSwap', []);
    // await deployer.verifyDeployment(meshSwapContract);
    
    console.log('Deployment completed successfully!');
  } catch (error) {
    console.error('Deployment failed:', error);
    process.exit(1);
  }
}

if (require.main === module) {
  main().catch(console.error);
}
