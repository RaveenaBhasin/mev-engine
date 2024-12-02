pub mod flashloan;
pub mod ekuboRouter;

pub mod interfaces {
    pub mod IFlashLoan;
    pub mod IVesu;
}

#[cfg(test)]
pub mod tests {
    pub mod test_flashloan;
    pub mod test_ekuboRouter;
}
