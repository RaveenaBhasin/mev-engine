pub mod flashloan;
pub mod shared_locker;
pub mod ekuboRouter;

pub mod interfaces {
    pub mod IFlashLoan;
    pub mod IVesu;
}

#[cfg(test)]
pub mod tests {
    pub mod test_flashloan;
}
