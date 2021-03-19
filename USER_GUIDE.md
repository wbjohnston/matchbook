# Matchbook User Guide

You can connect to Matchbook using any program that can connect to a TCP socket that supports TLS. In this guide we'll be using [`ncat`](https://nmap.org/ncat/) to connect

```bash
ncat --ssl localhost 8080
```

## Logging on

In order to initiate a fix session, you'll need to send a [Logon (A)](https://www.onixs.biz/fix-dictionary/4.2/msgtype_a_65.html) message to identify yourself to Matchbook. Matchbook currently will naively trust whatever id you pass in the `SenderCompId` field.

To logon we'll send this message.

```jsonc
{
    "Header": {
        "BeginString": "FIX.4.2", // matchbook only supports
        "MsgType": "A",
        "SenderCompID": "seller", // <- client id
        "TargetCompID": "matchbook", // we're sending this message to matchbook
        "MsgSeqNum": 1,
        "SendingTime": "2021-03-16 21:58:53.521981634 UTC"
    },
    "Body": {},
    "Trailer": {}
}
```

If we succesffuly authenticate, the logon message will be echoed back to you.

```jsonc
{
    "Header": {
        "BeginString": "FIX.4.2",
        "MsgType": "A",
        "SenderCompID": "matchbook", // note that the target and sender ids swapped now
        "TargetCompID": "seller",
        "MsgSeqNum": 1,
        "SendingTime": "2021-03-16T21:58:53.521981634Z"
    },
    "Body": {},
    "Trailer": {}
}
```

## Submitting an order

now that we're authenticated, we can start submitting orders.

```jsonc
{
    "Header": {
        "BeginString": "FIX.4.2",
        "MsgType": "D",
        "SenderCompID": "seller",
        "TargetCompID": "matchbook",
        // note that we need to increase the sequence number with every message we send
        "MsgSeqNum": 2, 
        "SendingTime": "2021-03-16 21:58:53.521981634 UTC"
    },
    "Body": {
        "ClOrdId": "foobar",
        "HandlInst": "3",
        "Price": 12.0,
        "Symbol": "ADBE",
        "Side": "1",
        "TransactTime": "2021-03-16 21:58:53.521981634 UTC",
        "OrdType": "2",
        "OrderQty": 100.0
    },
    "Trailer": {}
}
```

when the order is submitted successfully, you'll receive an [Execution Report (8)](https://www.onixs.biz/fix-dictionary/4.2/msgtype_8_8.html) message telling you that your message was submitted successfully.


```jsonc
{
    "Header": {
        "BeginString": "FIX.4.2",
        "MsgType": "8",
        "SenderCompID": "matchbook",
        "TargetCompID": "seller",
        "MsgSeqNum": 1,
        "SendingTime": "2021-03-19T20:35:03.363358261Z"
    },
    "Body": {
        "OrderID": "0",
        "OrdStatus": "0",
        "ExecTransType": "0",
        "ExecType": "0",
        "LeavesQty": 100.0,
        "CumQty": 0.0,
        "AvgPx": 0.0,
        "Symbol": "ADBE",
        "Side": "1",
        "OrderQty": 100.0
    },
    "Trailer": {}
}
```

When your order is executed, you'll receive another [Execution Report(8)](https://www.onixs.biz/fix-dictionary/4.2/msgtype_8_8.html) message with information about the execution.

```jsonc
{
    "Header": {
        "BeginString": "FIX.4.2",
        "MsgType": "8",
        "SenderCompID": "matchbook",
        "TargetCompID": "seller",
        "MsgSeqNum": 2,
        "SendingTime": "2021-03-19T20:38:23.324816793Z"
    },
    "Body": {
        "OrderID": "0",
        "OrdStatus": "0",
        "ExecID": "0",
        "ExecTransType": "0",
        "ExecType": "0",
        "LeavesQty": 0.0,
        "CumQty": 100.0,
        "AvgPx": 12.0,
        "Symbol": "ADBE",
        "Side": "1",
        "OrderQty": 100.0
    },
    "Trailer": {}
}
```
