# Support of progamming language

## Language supported

- [x] Java
- [x] C++
- [ ] XML
- [ ] TypeScript

## How to add a new language

It is for now **NOT** a simple task to add a language. The objective is it to be as simple as possible.

Unlike Github which only stores character additions and deletions in the source code (wich is not language dependant), HyperAST needs to know the semantics of the language to be able to analyse it.

The final objective is to be able to add a new language by only adding the grammar of the language and few more informations without having to modify a lot the code of the project.
