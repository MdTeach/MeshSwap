import { HashLock } from '@1inch/cross-chain-sdk'


function main() {
    let secret = "0x242b7a112ced4f1e688d117f358e3534e92f9e5fc89a5d0b2f843afebb9742f6"
    let expectedHash = "0xe7df6c631fad9c95bdf7e16b37d3f92184d59eb59155b3e60204b75c3b5984b3"
    let obtainedHash = HashLock.hashSecret(secret)
    
    if (obtainedHash !== expectedHash) {
        throw new Error(`❌ Hash mismatch: expected ${expectedHash}, got ${obtainedHash}`)
    } else {
        console.log("✅ Hash matches expected value. 🎉")
    }

}

main();