kill -9 $(lsof -t -i:8899)
solana-test-validator -r --quiet &
VALIDATOR_PID=$!
anchor keys sync
anchor build -- --features upgradable-test  
solana program deploy target/deploy/anchor_token_swap.so --program-id target/deploy/anchor_token_swap-keypair.json 
anchor run test-upgradable
kill -9 $(lsof -t -i:8899)

