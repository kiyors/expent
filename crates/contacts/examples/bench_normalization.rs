use std::time::Instant;

fn normalize_name(name: &str) -> String {
    // Simulated work
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    let names = vec![
        "John Doe",
        "Jane Smith",
        "Robert De Niro",
        "Al Pacino",
        "Christopher Nolan",
        "Quentin Tarantino",
        "Steven Spielberg",
        "Martin Scorsese",
        "Leonardo DiCaprio",
        "Brad Pitt",
    ];

    let iterations = 10000;
    let start = Instant::now();

    for _ in 0..iterations {
        for name in &names {
            let _ = normalize_name(name);
        }
    }

    let duration = start.elapsed();
    println!(
        "Time taken for {} iterations of {} names: {:?}",
        iterations,
        names.len(),
        duration
    );
    // names.len() is a tiny compile-time-known slice; u32::try_from cannot fail here.
    let names_len = u32::try_from(names.len()).expect("names.len() fits in u32");
    println!(
        "Average time per normalization: {:?}",
        duration / (iterations * names_len)
    );
}
