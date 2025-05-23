# fulgurite

### A simple test suit for Solana smart-contracts.

## Usage
Add this lines to Cargo.toml of your contract project. 
It will compile the contract for the test suit for your native machine, 
while keeping backward compatibility with Solana VM. 
There are no additional changes to the contract required.

```toml
[features]
default = []
no-entrypoint = []
inline = []

[target.'cfg(not(target_arch = "solana"))'.dependencies]
solana-program = {path = "../../solana-program"}
spl-token = {git = "https://github.com/Zarve8/fulgurite.git", features = ["no-entrypoint"]}
spl-associated-token-account = {git = "https://github.com/Zarve8/fulgurite.git", features = ["no-entrypoint"]}

[else]
solana-program = "=1.16.10"
spl-token = "3.5.0"
spl-associated-token-account = "1.1.3"
```

Include your contracts into test repository with with as in example.
```toml
fulgurite = {git = "https://github.com/Zarve8/fulgurite.git"}
your-contract = {path="path/your-contract", features = ["inline"]}
```

## Built-in Packages
Fulgurite suits comes with the following programmes: 
System Program, Spl Token Program, Spl Associated Program Account. 
If you need other packages you'd have to add them to the project and rebuild 
as a contract with system-program package substituted with fulgurite/system-program.

## Features
* Test with standard rust tests, multithreading supported
* Cross Program Invocation, PDA supported
* Account Datas & Logs interplay
* Lib and DLL contract import

## Limitations
* Account Data modification not checked
* Sysvar accounts not provided. Rent and Clock info provided directly
* No BPFLoader, use **Program** struct to import your contracts to the environment
* Other Solana programs cannot be import directly without sdk substitution