# LLM Proxy Architecture

This document provides a detailed overview of the LLM Proxy system architecture.

## System Overview

### High-Level Architecture

```text
                                    ┌─────────────────────┐
                                    │    Configuration    │
                                    │    (TOML Files)     │
                                    └─────────────────────┘
                                             │
                                             ▼
┌──────────┐    ┌─────────────┐    ┌─────────────────┐    ┌─────────────┐
│  Client  │───>│  HTTP       │───>│     Request     │───>│    LLM      │
│ Request  │    │  Server     │    │    Pipeline     │    │  Provider   │
└──────────┘    └─────────────┘    └─────────────────┘    └─────────────┘
                                             │                     │
                                             │                     │
                                    ┌─────────────────┐           │
                                    │    Response     │<──────────┘
                                    │     Stream      │
                                    └─────────────────┘
```

### Request-Response Flow

```text
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ Client   │     │ HTTP     │     │ Request  │     │ LLM      │     │ Response │
│ Request  │────>│ Server   │────>│ Pipeline │────>│ Provider │────>│ Handler  │
└──────────┘     └──────────┘     └──────────┘     └──────────┘     └──────────┘
     │                                 │                                    │
     │                                 │                                    │
     │                           ┌──────────┐                              │
     │                           │ Response │                              │
     └───────────────────────────│ Stream   │<─────────────────────────────┘
                                 └──────────┘
```

## Core Components

### 1. Request Pipeline Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Request Pipeline                         │
│                                                                │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌────────┐  │
│   │          │    │          │    │          │    │        │  │
│   │ Request  │───>│Processor │───>│Processor │───>│  LLM   │  │
│   │ Parser   │    │    1     │    │    2     │    │ Client │  │
│   │          │    │          │    │          │    │        │  │
│   └──────────┘    └──────────┘    └──────────┘    └────────┘  │
│        ▲               ▲               ▲              ▲        │
│        │               │               │              │        │
│   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐  │
│   │  Parser  │   │Processor │   │Processor │   │ Client   │  │
│   │  Config  │   │ Config 1 │   │ Config 2 │   │ Config  │  │
│   └──────────┘   └──────────┘   └──────────┘   └──────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Request Processing Stages

```text
┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐
│            │   │            │   │            │   │            │   │            │
│   Raw      │──>│  Parsed   │──>│ Processed  │──>│   Final   │──>│   LLM      │
│  Request   │   │  Request  │   │  Request   │   │  Request  │   │  Response  │
│            │   │            │   │            │   │            │   │            │
└────────────┘   └────────────┘   └────────────┘   └────────────┘   └────────────┘
      │                │                │                │                │
      ▼                ▼                ▼                ▼                ▼
┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐
│  Request   │   │ Validation │   │ Processing │   │  Provider  │   │  Response  │
│ Parsing    │   │   Rules    │   │   Rules    │   │   Rules    │   │ Processing │
└────────────┘   └────────────┘   └────────────┘   └────────────┘   └────────────┘
```

### 3. Component Interactions

```text
┌────────────────────────────────────────────────────────────┐
│                     Request Pipeline                       │
│                                                           │
│    ┌──────────┐          ┌──────────┐      ┌──────────┐  │
│    │ Request  │          │Processor │      │  LLM     │  │
│    │ Parser   │◄────────►│  Chain  │◄────►│ Client   │  │
│    └──────────┘          └──────────┘      └──────────┘  │
│         ▲                      ▲                ▲         │
└─────────┼──────────────────────┼────────────────┼─────────┘
          │                      │                │
    ┌──────────┐          ┌──────────┐     ┌──────────┐
    │  Token   │          │   URL    │     │ Config   │
    │ Provider │          │ Provider │     │ Provider │
    └──────────┘          └──────────┘     └──────────┘
```

### 4. Streaming Response Architecture

```text
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│            │    │            │    │            │    │            │
│   LLM     │───>│  Stream    │───>│  Channel   │───>│  Client    │
│ Provider  │    │ Processor  │    │  Buffer    │    │ Response   │
│            │    │            │    │            │    │            │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
       │               │                  │                 │
       ▼               ▼                  ▼                 ▼
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│  Chunk     │    │   Error    │    │ Backpressure│    │  Stream    │
│ Generation │    │ Handling   │    │  Control   │    │   Close    │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
```

## Configuration System

### Configuration Hierarchy

```text
┌───────────────────────────────────────────┐
│              config.toml                  │
│                                          │
│  ┌────────────┐        ┌────────────┐    │
│  │  Server    │        │  Global    │    │
│  │  Config    │        │  Config    │    │
│  └────────────┘        └────────────┘    │
│                                          │
│  ┌────────────┐        ┌────────────┐    │
│  │  Provider  │        │ Processor  │    │
│  │  Configs   │        │  Configs   │    │
│  └────────────┘        └────────────┘    │
│                                          │
│  ┌────────────┐        ┌────────────┐    │
│  │   Route    │        │  Security  │    │
│  │  Configs   │        │   Config   │    │
│  └────────────┘        └────────────┘    │
└───────────────────────────────────────────┘
```

### Configuration Flow

```text
┌────────────┐     ┌────────────┐     ┌────────────┐
│            │     │            │     │            │
│  Config    │────>│  Config    │────>│ Component  │
│   File     │     │  Parser    │     │   Init     │
│            │     │            │     │            │
└────────────┘     └────────────┘     └────────────┘
      │                  │                  │
      ▼                  ▼                  ▼
┌────────────┐     ┌────────────┐     ┌────────────┐
│Environment │     │  Default   │     │ Runtime    │
│ Variables  │     │  Values    │     │ Validation │
└────────────┘     └────────────┘     └────────────┘
```

## Security Model

### Authentication Flow

```text
┌────────────┐     ┌────────────┐     ┌────────────┐     ┌────────────┐
│            │     │            │     │            │     │            │
│  Client    │────>│   Auth     │────>│  Token    │────>│   LLM      │
│  Request   │     │ Middleware │     │ Provider  │     │  Provider  │
│            │     │            │     │            │     │            │
└────────────┘     └────────────┘     └────────────┘     └────────────┘
                         │                  │
                         ▼                  ▼
                   ┌────────────┐     ┌────────────┐
                   │   Rate     │     │   Token    │
                   │  Limiting  │     │  Storage   │
                   └────────────┘     └────────────┘
```

### Request Validation

```text
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│            │    │            │    │            │    │            │
│  Input     │───>│  Schema    │───>│  Content  │───>│  Security  │
│ Validation │    │ Validation │    │ Validation│    │   Checks   │
│            │    │            │    │            │    │            │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
       │                │                 │                 │
       ▼                ▼                 ▼                 ▼
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│  Type      │    │  Field     │    │  Size      │    │  Token     │
│  Checks    │    │  Checks    │    │  Limits    │    │  Checks    │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
```

## Error Handling

### Error Propagation

```text
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│            │    │            │    │            │    │            │
│  Error     │───>│  Error     │───>│  Error     │───>│  Client    │
│  Source    │    │ Handling   │    │  Logging   │    │ Response   │
│            │    │            │    │            │    │            │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
       │                │                 │                 │
       ▼                ▼                 ▼                 ▼
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│  Error     │    │  Recovery  │    │  Metrics   │    │  Status    │
│  Context   │    │  Actions   │    │ Collection │    │   Code     │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
```

## Monitoring System

### Logging Architecture

```text
┌────────────────────────────────────────────────────────┐
│                   Logging System                       │
│                                                       │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐        │
│  │ Request  │    │ Process  │    │ Response │        │
│  │  Logs    │───>│  Logs   │───>│   Logs   │        │
│  └──────────┘    └──────────┘    └──────────┘        │
│       │              │               │                │
│       ▼              ▼               ▼                │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐        │
│  │  Error   │    │ Metrics  │    │  Trace   │        │
│  │  Logs    │    │          │    │   Logs   │        │
│  └──────────┘    └──────────┘    └──────────┘        │
└────────────────────────────────────────────────────────┘
```

### Metrics Collection

```text
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│            │    │            │    │            │    │            │
│  Request   │───>│ Processing │───>│  Provider  │───>│  Response  │
│  Metrics   │    │  Metrics  │    │  Metrics   │    │  Metrics   │
│            │    │            │    │            │    │            │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
       │                │                 │                 │
       ▼                ▼                 ▼                 ▼
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│  Latency   │    │   Memory   │    │    API     │    │  Status    │
│  Metrics   │    │   Usage    │    │   Stats    │    │   Codes    │
└────────────┘    └────────────┘    └────────────┘    └────────────┘
```
