(module
  (import "env" "require" (func $require (param i32)))
  (import "env" "wasm_input" (func $wasm_input (param i32) (result i64)))
  (func $read_public_input (result i64)
    i32.const 1
    call $wasm_input
  )
  (func $read_private_input (result i64)
    i32.const 0
    call $wasm_input
  )
  (func $fibonacci (param $var0 i64) (result i64)
    (local $var1 i32) (local $var2 i32) (local $var3 i32) (local $var4 i64) 
    i64.const 1
    local.set $var4
    block $label0
      local.get $var0
      i64.const 1
      i64.lt_s
      br_if $label0
      block $label1
        local.get $var0
        i64.const 1
        i64.eq
        br_if $label1
        local.get $var0
        i64.const -1
        i64.add
        local.set $var0
        i32.const 0
        local.set $var3
        i32.const 1
        local.set $var2
        loop $label2
          local.get $var2
          local.tee $var1
          local.get $var3
          i32.add
          i64.extend_s/i32
          i64.const 100
          i64.rem_s
          local.tee $var4
          i32.wrap/i64
          local.set $var2
          local.get $var1
          local.set $var3
          local.get $var0
          i64.const -1
          i64.add
          local.tee $var0
          i64.const 0
          i64.ne
          br_if $label2
        end $label2
      end $label1
      local.get $var4
      return
    end $label0
    i64.const 0
  )
  (func $zkmain (result i32)
    call $read_public_input  ;; Call input n and push it onto the stack
    call $fibonacci          ;; Call fibonacci(n)
    call $read_public_input  ;; Call to get public input m and push it onto the stack
    i64.eq                   ;; Check if fibonacci(n) == m
    call $require            ;; Call require with the result of the equality check
    i32.const 0              ;; Return 0
  )

  (export "zkmain" (func $zkmain))
)
