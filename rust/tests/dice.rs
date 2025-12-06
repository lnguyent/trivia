use trivia::Dice;

#[test]
fn test_dice_can_be_created_from_i32() {
    let dice1 = Dice::from(1);
    let dice2 = Dice::from(2);
    let dice3 = Dice::from(3);
    let dice4 = Dice::from(4);
    let dice5 = Dice::from(5);
    let dice6 = Dice::from(6);
    assert_eq!(dice1, Dice::One);
    assert_eq!(dice2, Dice::Two);
    assert_eq!(dice3, Dice::Three);
    assert_eq!(dice4, Dice::Four);
    assert_eq!(dice5, Dice::Five);
    assert_eq!(dice6, Dice::Six);
}

#[test]
fn test_dice_can_be_created_from_usize() {
    let dice1 = Dice::from(1);
    let dice2 = Dice::from(2);
    let dice3 = Dice::from(3);
    let dice4 = Dice::from(4);
    let dice5 = Dice::from(5);
    let dice6 = Dice::from(6);
    assert_eq!(dice1, Dice::One);
    assert_eq!(dice2, Dice::Two);
    assert_eq!(dice3, Dice::Three);
    assert_eq!(dice4, Dice::Four);
    assert_eq!(dice5, Dice::Five);
    assert_eq!(dice6, Dice::Six);
}

#[test]
fn test_when_invalid_value_then_panic() {
    let result = std::panic::catch_unwind(|| {
        let _ = Dice::from(0);
    });
    assert!(result.is_err());

    let result = std::panic::catch_unwind(|| {
        let _ = Dice::from(7);
    });
    assert!(result.is_err());
}
#[test]
fn test_dice_can_be_converted_to_usize() {
    for i in 1..=6 {
        let dice = Dice::from(i);
        let value: usize = (&dice).into();
        assert_eq!(value, i as usize);
    }
}

#[test]
fn test_dice_is_odd() {
    let dice1 = Dice::from(1);
    let dice2 = Dice::from(2);
    let dice3 = Dice::from(3);
    let dice4 = Dice::from(4);
    let dice5 = Dice::from(5);
    let dice6 = Dice::from(6);
    assert!(dice1.is_odd());
    assert!(!dice2.is_odd());
    assert!(dice3.is_odd());
    assert!(!dice4.is_odd());
    assert!(dice5.is_odd());
    assert!(!dice6.is_odd());
}
