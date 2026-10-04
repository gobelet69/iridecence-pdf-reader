with open("src/main.rs", "r") as f:
    main = f.read()

main = main.replace(
    "let mut rect = proposed_effective_rect;\n            // Expand the invisible hit area by 5 points on each side",
    "let mut rect = proposed_effective_rect;\n            println!(\"HIT AREA CALLED: {:?}\", proposed_effective_rect);\n            // Expand the invisible hit area by 5 points on each side"
)

with open("src/main.rs", "w") as f:
    f.write(main)
