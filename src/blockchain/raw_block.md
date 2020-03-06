# Block complete structure: bytes are serialized as following:

- hashed data
    - meta
        - version: **u8**
        - previous hash
            - length: **u8**
            - value: **[u8; 32]**
        - sequence: **u64**
        - user fields
            > count: **u8**
            >> user field content
            >> - key: **u32**
            >> - value content
            >>     - value type: **u8**
            >>      - value data is on of:
            >>          1. **u64** in case *type == 1*
            >>          2. **vec** in case *type == 2*, contains
            >>              - len: **u32**
            >>              - data: **[u8; len]**
            >>          3. **money** in case *type == 3*, contains
            >>              - integral: **i32**
            >>              - fraction: **u64**

        - round cost, money:
            - integral: **i32**
            - fraction: **u64**

    - transactions
        > count: **u32**
        >> - transaction content
        >>      - inner ID:
        >>          - lo: **u16**
        >>          - hi: **u32**
        >> - source is one of:
        >>>    - wallet id **u32** in case of (hi & 0x8000_0000) != 0
        >>>    -  PublicKey: **[u8; 32]**
        >>  - target is one of:
        >>>    - wallet id **u32** in case of (hi & 0x4000_0000) != 0
        >>>    - PublicKey: **[u8; 32]**
        >> - sum, money:
        >>>    - integral: **i32**
        >>>    - fraction: **u64**
        >> - max fee: **u16**
        >> - currency: **u8**
        >>- user fields
        >>>- count_uf: **u8**
        >>>>- user field content
        >>>>- key: **u32**
        >>>>- user field value
        >>>>    - value type: **u8**
        >>>>    - value data is one of:
        >>>>        1. **u64** in case of *type == 1*
        >>>>        2. **vec** in case of *type == 2*
        >>>>            - **len**: **u32**
        >>>>            - data: **[u8; len]**
        >>>>        3. **money** in case of *type == 3*
        >>>>            - integral: **i32**
        >>>>            - fraction: **u64**
        >>- transaction signature: **[u8; 64]**
        >>- transaction actual fee: **u16**

    - introduced new wallets:
        >- count_nw: **u32**
        >>- new wallet info
        >>      - address id: **u64** interpreted as
        >>          - *low 63 bits* - transaction index in block
        >>          - *hi 1 bit* - source "0" / target "1"
        >>      - wallet id: **u32**

    - trusted nodes info (round table):
        >- **count**: **u8**
        >- actual trusted: **u64** as bitset, count of "1" = **sig_blk_cnt**
        >- keys: **[[u8; 32]; count]**
        >- previous round table:
        >   - count_prev_rt: **u8**, **must** be equal to previous_block :: trusted_info :: count
        >   - actual_prev_rt: **u64** as bitset, count of "1" -> **sig_prev_rt_cnt**
        >   - signatures: **[[u8; 64]; sig_prev_rt_cnt]**
    - hashed len: **usize**

- signatures: **[[u8; 64]; sig_blk_cnt]**
    
- contract signatures:
    >- count_contract_sig: **u8**
    >>- contract_sig_data
    >>  - key: **[u8; 32]**
    >>  - round: **u64**
    >>> - trusted_cnt: **u8**
    >>>    - trusted_idx: **u8**
    >>>    - trusted_sig: **[u8; 64]**
