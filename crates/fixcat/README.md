# fixcat

``` shell
Pretty print FIX protocol messages

Usage: fixcat [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Optional input files, if not specifed input will be read from STDIN

Options:
      --admin            Include administrative messages
      --mix              Print non FIX text in the output
      --orders           Track order state
      --fields <FIELDS>  Comma separated list of field names or tags to display when tracking order state
  -h, --help             Print help
  -V, --version          Print version
```

fixcat is modelled on the UNIX cat utility; it will print FIX messages in human readable format with message, field, and enumerated value descriptions.

It will read from standard input or a list of one or more filenames specified on the command line.

``` shell
less fixlog | fixcat
fixcat fixlog1.txt fixlog2.txt fixlog3.txt
```

The input is expected to be raw FIX messages, one per line, with an optional prefix which would typically be a timestamp etc as is typical in log files. The parser searches for **8=FIX.** and assumes that is the start of the message which continues until a CheckSum field or end of line is reached.

```
8=FIX.4.49=7235=A49=INITIATOR56=ACCEPTOR34=152=20190816-10:34:27.75298=0108=3010=013

2019-08-17 13:00:00.000 8=FIX.4.49=7235=A49=INITIATOR56=ACCEPTOR34=152=20190816-10:34:27.75298=0108=3010=013
```

Parsing either of these lines with fixcat will product the following output.

```
Logon
{
     BeginString (   8) FIXT.1.1
      BodyLength (   9) 79
         MsgType (  35) A - Logon
    SenderCompID (  49) INITIATOR
    TargetCompID (  56) ACCEPTOR
       MsgSeqNum (  34) 1
     SendingTime (  52) 20200215-05:52:57.097
   EncryptMethod (  98) 0 - None
      HeartBtInt ( 108) 30
DefaultApplVerID (1137) 9
        CheckSum (  10) 159
}
```

By default any input lines that are not recognised as FIX data are discarded, these lines can be included with the `--mix` option.

The `--orders` option will track the state of any orders encountered in the input and print a table of all known orders whenever one of them changes. 

```
+------------------------------------------------------------------------------------------------------------------------------------------------+
| SenderCompID   TargetCompID   ClOrdID   OrigClOrdID   Symbol   OrdStatus   OrdType   TimeInForce      Side   OrderQty   Price   CumQty   AvgPx |
+------------------------------------------------------------------------------------------------------------------------------------------------+
| INITIATOR      ACCEPTOR       7                       WTF.AX   Replaced    Limit     GoodTillCancel   Buy       20000   11.56        0       0 |
| INITIATOR      ACCEPTOR       8         7             WTF.AX   Replaced    Limit     GoodTillCancel   Buy       40000   11.58    10000   11.58 |
| INITIATOR      ACCEPTOR       9         8             WTF.AX   Canceled    Limit     GoodTillCancel   Buy       35000   11.58    15000   11.58 |
+------------------------------------------------------------------------------------------------------------------------------------------------+
```

The columns in this table can be specified with the `--fields` option. This is a comma separated list of field names or tag values.

```
$ ./fixcat --orders --fields "OrdStatus,100,Side,39,40,Price,CumQty,AvgPx" < fix.log

<SNIP>

+---------------------------------------------------------------------------------+
| OrdStatus   ExDestination   Side   OrdStatus   OrdType   Price   CumQty   AvgPx |
+---------------------------------------------------------------------------------+
| Replaced    AUTO            Buy    Replaced    Limit     11.56        0       0 |
| Replaced    AUTO            Buy    Replaced    Limit     11.58    10000   11.58 |
| Canceled    AUTO            Buy    Canceled    Limit     11.58    15000   11.58 |
+---------------------------------------------------------------------------------+
```

The fields can also be specified via the environment variable `CROCOFIX_FIXCAT_FIELDS`.

```
$ export CROCOFIX_FIXCAT_FIELDS="OrdStatus,100,Side,39,40,Price,CumQty,AvgPx"
$ ./fixcat --orders < fix.log

<SNIP>

+---------------------------------------------------------------------------------+
| OrdStatus   ExDestination   Side   OrdStatus   OrdType   Price   CumQty   AvgPx |
+---------------------------------------------------------------------------------+
| Replaced    AUTO            Buy    Replaced    Limit     11.56        0       0 |
| Replaced    AUTO            Buy    Replaced    Limit     11.58    10000   11.58 |
| Canceled    AUTO            Buy    Canceled    Limit     11.58    15000   11.58 |
+---------------------------------------------------------------------------------+
```

The `--fields` option takes precendence over `CROCOFIX_FIXCAT_FIELDS`.