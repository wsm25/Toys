use super::*;

#[test]
fn insert_get_remove_round_trip() {
    let mut colony = Colony::new();
    let handle = colony.insert("alpha");
    assert_eq!(colony.get(handle), Some(&"alpha"));
    assert_eq!(colony.remove(handle), Some("alpha"));
    assert_eq!(colony.get(handle), None);
    assert!(colony.is_empty());
}

#[test]
fn removed_slot_is_reused_with_new_generation() {
    let mut colony = Colony::new();
    let first = colony.insert(10);
    assert_eq!(colony.remove(first), Some(10));

    let second = colony.insert(20);
    assert_eq!(first.group(), second.group());
    assert_eq!(first.slot(), second.slot());
    assert_ne!(first.generation(), second.generation());
    assert_eq!(colony.get(first), None);
    assert_eq!(colony.get(second), Some(&20));
}

#[test]
fn iteration_skips_holes() {
    let mut colony = Colony::new();
    let a = colony.insert(1);
    let _b = colony.insert(2);
    let c = colony.insert(3);
    colony.remove(a);
    colony.remove(c);

    let values: Vec<_> = colony.iter().copied().collect();
    assert_eq!(values, vec![2]);
}

#[test]
fn iter_mut_updates_in_place() {
    let mut colony = Colony::new();
    colony.insert(1);
    colony.insert(2);
    colony.insert(3);

    for value in &mut colony {
        *value *= 10;
    }

    let values: Vec<_> = colony.iter().copied().collect();
    assert_eq!(values, vec![10, 20, 30]);
}

#[test]
fn iter_mut_updates_across_groups_with_holes() {
    let mut colony = Colony::new();
    let mut handles = Vec::new();
    for value in 0..(GROUP_LEN as i32 + 4) {
        handles.push(colony.insert(value));
    }

    colony.remove(handles[1]);
    colony.remove(handles[GROUP_LEN - 1]);
    colony.remove(handles[GROUP_LEN]);
    colony.remove(handles[GROUP_LEN + 2]);

    for value in &mut colony {
        *value += 1000;
    }

    let values: Vec<_> = colony.iter().copied().collect();
    let expected: Vec<_> = (0..(GROUP_LEN as i32 + 4))
        .filter(|&value| {
            value != 1
                && value != (GROUP_LEN as i32 - 1)
                && value != GROUP_LEN as i32
                && value != (GROUP_LEN as i32 + 2)
        })
        .map(|value| value + 1000)
        .collect();
    assert_eq!(values, expected);
}

#[test]
fn iterates_full_group_fast_path() {
    let mut colony = Colony::new();
    for value in 0..GROUP_LEN {
        colony.insert(value);
    }

    let values: Vec<_> = colony.iter().copied().collect();
    assert_eq!(values, (0..GROUP_LEN).collect::<Vec<_>>());
}

#[test]
fn with_capacity_preallocates_groups() {
    let colony = Colony::<usize>::with_capacity(GROUP_LEN + 1);

    assert_eq!(colony.len(), 0);
    assert_eq!(colony.capacity(), GROUP_LEN * 2);
}

#[test]
fn with_capacity_inserts_from_first_preallocated_group() {
    let mut colony = Colony::with_capacity(GROUP_LEN + 1);
    let first = colony.insert(10);
    let second_group = (1..=GROUP_LEN)
        .map(|value| colony.insert(value))
        .last()
        .unwrap();

    assert_eq!(first.group(), 0);
    assert_eq!(first.slot(), 0);
    assert_eq!(second_group.group(), 1);
    assert_eq!(second_group.slot(), 0);
    assert_eq!(colony.capacity(), GROUP_LEN * 2);
}

#[test]
fn iterates_zst_values() {
    let mut colony = Colony::new();
    for _ in 0..(GROUP_LEN + 3) {
        colony.insert(());
    }

    assert_eq!(colony.iter().count(), GROUP_LEN + 3);
    assert_eq!(colony.iter_mut().count(), GROUP_LEN + 3);
}

#[test]
fn iterates_contiguous_runs_after_holes() {
    let mut colony = Colony::new();
    let handles: Vec<_> = (0..12).map(|value| colony.insert(value)).collect();

    colony.remove(handles[0]);
    colony.remove(handles[4]);
    colony.remove(handles[5]);
    colony.remove(handles[9]);

    let values: Vec<_> = colony.iter().copied().collect();
    assert_eq!(values, vec![1, 2, 3, 6, 7, 8, 10, 11]);
}

#[test]
fn iterator_fold_sums_values() {
    let mut colony = Colony::new();
    for value in 0..10 {
        colony.insert(value);
    }

    assert_eq!(colony.iter().sum::<i32>(), 45);
}

#[test]
fn iterator_mut_fold_updates_values() {
    let mut colony = Colony::new();
    for value in 0..4 {
        colony.insert(value);
    }

    colony.iter_mut().fold((), |(), value| {
        *value += 10;
    });

    let values: Vec<_> = colony.iter().copied().collect();
    assert_eq!(values, vec![10, 11, 12, 13]);
}

#[test]
fn remove_from_full_group_makes_it_reusable_without_scanning() {
    let mut colony = Colony::new();
    let mut handles = Vec::new();
    for value in 0..(GROUP_LEN as i32 * 2) {
        handles.push(colony.insert(value));
    }

    let recycled = handles[GROUP_LEN / 2];
    assert_eq!(colony.remove(recycled), Some((GROUP_LEN / 2) as i32));

    let replacement = colony.insert(9_999);
    assert_eq!(replacement.group(), recycled.group());
    assert_eq!(replacement.slot(), recycled.slot());
    assert_ne!(replacement.generation(), recycled.generation());
}
