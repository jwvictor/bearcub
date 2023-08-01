# Bearcub: storage for Cubby

Bearcub is an efficient backend storage engine for [Cubby](https://github.com/jwvictor/cubby). It is being built for use with the forthcoming modular backend architecture in Cubby.

The storage engine is unauthenticated -- it is intended to be called _only_ from the Cubby server, which functions as the final arbiter of authentication and authorization questions.

Bearcub uses an efficient, minimal, and compact binary protocol for exchanging messages between the Cubby server and the engine (details below under 'Wire Format). Each message is split into frames, and each frame has a type code. Most messages are a single frame, but messages that send large data blobs can be more than that.

## Wire format

### Message Types

```
 Code   Type
 G      Get by ID
 P      Get by prefix
 p      Put data
 s      Set data
 d      Continued data frame
```

### General Layout

```
 Byte   Format      Contents
 0-3    Char        ASCII 'c0.1' 
 4-7    32-bit int  Frame length
 8-11   32-bit int  Number of frames remaining in message (including this one)
 12     Char        Message type
 13+    Binary      Message data
```

### Read Instructions (G, P)

```
 Byte   Format      Contents
 13-48  Char        User UUID
 49+    Char        Key
```

### Write Instuctions (p, s)

```
 Byte   Format      Contents
 13-48  Char        User UUID
 49-84  Char        UUID
 85-121 Char        Parent UUID (0s for root)
 121+   Bin         Data (JSON)
```

### Continued Data Instruction (d)

```
 Byte   Format      Contents
 13+    Char        Data 
```

