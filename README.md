# SFL Explorer: An Interactive Tool for Learning Functional Langauges
SFL Explorer (https://functional.kiransturt.co.uk/) is an open-source educational platform designed to demystify the internal mechanics of functional programming. By providing a transparent, step-by-step visualization of term reduction, it helps students and developers build an intuitive mental model of how high-level functional code is transformed into a final result. 

As the system is designed to be a teaching tool, I have done user testing, on a total of 27 students. This took the form of 3 focus groups at various points throughout the project, with students who are at various stages in the journey of learning functional languages. Their feedback ensured that the project stayed on track and remained as useful as possible to potential users with a wide variety of skill levels.

For a deep dive into the formal semantics, type system implementation, and the bidirectional checking algorithm, please refer to the full Dissertation PDF found at the repo root.

SFL utilizes bidirectional typechecking to ensure that all programs are well-typed before evaluation, preventing runtime type errors. It supports ADTs and polymorphism.

## The Language: Simple Functional Language (SFL)
SFL is a minimal, generic functional language designed for clarity. While simple, it implements the sophisticated features found in "industrial" languages like Haskell or OCaml:
- Polymorphism & ADTs: Support for user-defined algebraic data types and pattern matching.
- Bidirectional Typechecking: Based on the Dunfield & Krishnaswami algorithm, the system provides clear feedback during the type-checking phase to help users understand type soundness.
- Familiar Syntax: Designed, with help from student focus groups, to be easily transferable to other functional languages.
- A prelude: Optional, but included by default. Provides common functional langauge patterns like `map` and `fold`: [prelude.sfl](prelude.sfl).

## Example SFL Code and Evaluation
pairs
```
pair :: a -> b -> (a, b)
pair x y = (x, y)

first :: (a, b) -> a
first (x, _) = x

second :: (a, b) -> b
second (_, x) = x

main :: Bool
main = second $ (pair 1 true)
```
Evaluation (read bottom to top)
<img width="1206" height="741" alt="image" src="https://github.com/user-attachments/assets/228e3275-54a0-4abc-bc23-7cb7cadecd5d" />

## Usage
On the website you may enter a program. By default the program includes the Prelude, but this can be turned off in settings. Many default programs are included for your conveniance. 

You may choose between lazy and free choice evaluation. Free choice allows you to pick from any of the valid next steps (i.e. expressions to reduce).

## Typechecking

Based on "Complete and Easy Bidirectional Typechecking for Higher-Rank Polymorphism" [Jana Dunfield, Neelakantan R. Krishnaswami] (referenced as 8 in the below algorithm preview). See section 4.3.5 of my ![Dissertation](Dissertation.pdf) for a full explaination and an example derivation. 
### Preview
Below is a preview of the typechecking algorithm, taken from Appendix E.

Typechecking             | Algorithmic Subtyping and Context Instantiation
:-------------------------:|:-------------------------:
<img width="1106" height="1027" alt="image" src="https://github.com/user-attachments/assets/02fa9f52-0d13-4d4b-b482-eaedfdcc2549" /> | <img width="700" height="970" alt="image" src="https://github.com/user-attachments/assets/8ff31b3c-ebae-4b67-a09f-aae0f87c9566" />

## The Website
All functionality for the language is written in Rust. The Rust functionality is compiled to Web Assembly,
and included into a React app that acts as the frontend. This functionality is therefore available entirely client
side, requiring no client-server interaction. The app is a Progressive Web App (PWA) and is able to be installed
and used offline.
