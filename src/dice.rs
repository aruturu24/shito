use rand::Rng;

pub fn roll(dice: &str, modifier: i32) -> (i32, Vec<i32>) {
    // Very simple parser: NdM or dM where N default 1
    // Supports advantage/disadvantage with "2d20kh1" or "2d20kl1" (keep high/low 1)
    // For simplicity here: support NdM only
    let (count, sides) = parse_dice(dice).unwrap_or((1, 20));
    let mut rng = rand::thread_rng();
    let mut rolls = Vec::with_capacity(count as usize);
    let mut total = 0;
    for _ in 0..count {
        let r = rng.gen_range(1..=sides);
        rolls.push(r);
        total += r;
    }
    (total + modifier, rolls)
}

pub fn parse_dice(spec: &str) -> Option<(i32, i32)> {
    let s = spec.trim().to_lowercase();
    let parts: Vec<&str> = s.split('d').collect();
    if parts.len() != 2 { return None; }
    let count = if parts[0].is_empty() { 1 } else { parts[0].parse().ok()? };
    let sides = parts[1].parse().ok()?;
    if count <= 0 || sides <= 0 { return None; }
    Some((count, sides))
}
