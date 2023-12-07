#include "foreign.h"

void poseidon(uint64_t* data, uint32_t size, uint64_t* r)
{
    int i;
    poseidon_new(size);
    for(i=0; i<size; i++) {
        uint64_t* a = data[i];
        poseidon_push(*a);
    }
    r[0] = poseidon_finalize();
    r[1] = poseidon_finalize();
    r[2] = poseidon_finalize();
    r[3] = poseidon_finalize();
    wasm_dbg(r[0]);
    wasm_dbg(r[1]);
    wasm_dbg(r[2]);
    wasm_dbg(r[3]);
}

int zkmain(){
    int size = 32;
    uint64_t data[size];
    uint64_t r[size];
    for (uint64_t i=0; i<size; i++){
        data[i] = i+1;
    }
    poseidon(&data, size, &r);
    return 0;
}