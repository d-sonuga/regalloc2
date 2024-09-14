use std::fs;

fn main() {
    let file = fs::read_to_string("edits-measure").unwrap();
    let (mut total_edits_len_guess, mut total_actual_edits_len, mut total_num_insts) = (0, 0, 0);
    let mut iter = file.split("\n");
    let mut no_of_rows = 0;
    let mut num_insts_actual_edits_gt = vec![];
    let mut edits_to_num_insts_ratios = vec![];
    iter.next();
    iter.next();
    for row in iter {
        if row.trim().is_empty() {
            continue;
        }
        let (_, num_insts, edits_len_guess, actual_edits_len) = parse(row);
        total_actual_edits_len += actual_edits_len;
        total_num_insts += num_insts;
        total_edits_len_guess += edits_len_guess;
        no_of_rows += 1;
        if actual_edits_len > num_insts {
            num_insts_actual_edits_gt.push((num_insts, actual_edits_len));
            edits_to_num_insts_ratios.push(actual_edits_len / num_insts);
        }
    }
    println!("total actual edits len: {}, total num insts: {}", total_actual_edits_len, total_num_insts);
    println!("total edits len guess: {}", total_edits_len_guess);
    println!("ave. actual edits per inst: {}", total_actual_edits_len / total_num_insts);
    println!("no rows with num insts < actual edits: {}", num_insts_actual_edits_gt.len());
    println!("total num of rows: {}", no_of_rows);
    println!("max edits to num_insts_ratio: {:?}", edits_to_num_insts_ratios.iter().max());
    println!("ave. edits to num_insts ratio: {}", edits_to_num_insts_ratios.iter().sum::<usize>() / edits_to_num_insts_ratios.len());
}

fn parse(row: &str) -> (usize, usize, usize, usize) {
    let mut iter = row.split(" ");
    (
        iter.next().unwrap().parse().unwrap(),
        iter.next().unwrap().parse().unwrap(),
        iter.next().unwrap().parse().unwrap(),
        iter.next().unwrap().parse().unwrap(),
    )
}
