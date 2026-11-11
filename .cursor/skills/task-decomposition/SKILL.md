---
name: task-decomposition
description: Splits a request into single-meaning parts before recording tasks and before calling several agents. Use when the work is wider than one known edit.
disable-model-invocation: true
---

# Decomposition

## Purpose

Cut the request so each part has one outcome and one writer.

## Trigger

New feature, several layers, several files, words «продолжай» and «делай» on work not yet done.

## Input

User wording, docs already read, known paths.

## Who

Team Lead. Recording parts — skill `tasks`.

## Steps

1. Separate a point edit of a known file from the rest. Keep the point edit yourself.
2. Cut the rest by meaning: one contract, one screen, one schema. Do not put a neighboring layer into the same part unless it was named.
3. For each part, record goal, paths, what is already known, and which file will be the answer.
4. Mark parts that write the same files as a queue. Independent reads may run in parallel.
5. If a cut lacks a fact, stop on one question. Do not fill the hole with an invented field or route.

## Result

Part list. Each has one meaning, an owner, and file bounds.

## Quality

A part can go to an agent without the whole chat. Two parts do not edit one file at once.

## Limit

Do not assign every employee to every part. Narrow the roster when a fact already exists.

## Example

“Add notifications” splits into contract, server write, and UI display. That is three parts, not one “do everything” prompt.
