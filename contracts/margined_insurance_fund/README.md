# Margined Protocol Insurance Fund

Insurance fund contains funds that are used to cover shortfall(slippage) in funding payments, additionally insurance fund accrues a portion of transaction fees and profits.

---

## InstantiateMsg

The instantiation message is empty.

```json
{}
```

## ExecuteMsg

### `update_owner`

Enables transfer of contract ownership and the beneficiary of insurance funds. Beneficiary is an address that is able to request funds held by the insurance fund contract.

```json
{
   "update_owner": {
        "owner": "orai..."
        "beneficiary": "orai..."
   }
}
```

### `add_vamm`

Append vamm to list of supported vAMMs.

```json
{
  "add_vamm": {
    "vamm": "orai..."
  }
}
```

### `remove_vamm`

Remove vamm from list of supported vAMMs.

```json
{
  "remove_vamm": {
    "vamm": "orai..."
  }
}
```

### `withdraw`

Enables the beneficiary to request contract funds.

```json
{
  "withdraw": {
    "token": "orai...",
    "amount": "100"
  }
}
```

### `shutdown_vamms`

Emergency shutdown function that halts all vAMMs trading.

```json
{
  "shutdown_vamms": {}
}
```

## QueryMsg

### `config`

Returns contract parameters.

```json
{
  "config": {}
}
```

### `is_vamm`

Returns bool showing if vamm is supported.

```json
{
  "is_vamm": {
    "vamm": "orai..."
  }
}
```

### `get_all_vamm`

Returns list of supported vAMMs.

```json
{
    "get_all_vamm": {
        "limit"?: 69,
    }
}
```

### `get_all_vamm_status`

Returns the status of all vAMMs supported.

```json
{
    "get_all_vamm_status": {
        "limit"?: 69,
    }
}
```

### `get_vamm_status`

Returns the status of a specific vAMM.

```json
{
  "get_vamm_status": {
    "vamm": "orai..."
  }
}
```
