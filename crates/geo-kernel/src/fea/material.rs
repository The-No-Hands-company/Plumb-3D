//! Linear Elastic Material Properties & Constitutive Hooke's Law
//!
//! Provides isotropic material models, standard engineering material presets,
//! and 3D elasticity tensors for finite element analysis.

use serde::{Deserialize, Serialize};

/// Standard Engineering Material Preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MaterialPreset {
    #[default]
    StructuralSteel,
    Aluminum6061,
    TitaniumTi6Al4V,
    PlaPlastic,
    AbsPlastic,
    Custom,
}

/// Isotropic Linear Elastic Material Properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaMaterial {
    pub name: String,
    /// Young's Modulus E in Gigapascals (GPa)
    pub youngs_modulus_gpa: f32,
    /// Poisson's Ratio nu in [0.0, 0.499]
    pub poissons_ratio: f32,
    /// Yield Strength in Megapascals (MPa)
    pub yield_strength_mpa: f32,
    /// Density rho in kg/m^3
    pub density_kg_m3: f32,
}

impl Default for FeaMaterial {
    fn default() -> Self {
        Self::structural_steel()
    }
}

impl FeaMaterial {
    /// ASTM A36 Structural Steel
    pub fn structural_steel() -> Self {
        Self {
            name: "Structural Steel".to_string(),
            youngs_modulus_gpa: 210.0,
            poissons_ratio: 0.30,
            yield_strength_mpa: 250.0,
            density_kg_m3: 7850.0,
        }
    }

    /// Aluminum 6061-T6 Aerospace Alloy
    pub fn aluminum_6061() -> Self {
        Self {
            name: "Aluminum 6061-T6".to_string(),
            youngs_modulus_gpa: 68.9,
            poissons_ratio: 0.33,
            yield_strength_mpa: 276.0,
            density_kg_m3: 2700.0,
        }
    }

    /// Titanium Ti-6Al-4V Grade 5
    pub fn titanium_ti6al4v() -> Self {
        Self {
            name: "Titanium Ti-6Al-4V".to_string(),
            youngs_modulus_gpa: 113.8,
            poissons_ratio: 0.34,
            yield_strength_mpa: 880.0,
            density_kg_m3: 4430.0,
        }
    }

    /// PLA Filament (Polylactic Acid 3D Print)
    pub fn pla_plastic() -> Self {
        Self {
            name: "PLA Plastic (3D Print)".to_string(),
            youngs_modulus_gpa: 3.5,
            poissons_ratio: 0.36,
            yield_strength_mpa: 50.0,
            density_kg_m3: 1240.0,
        }
    }

    /// ABS Injection/Print Plastic
    pub fn abs_plastic() -> Self {
        Self {
            name: "ABS Plastic".to_string(),
            youngs_modulus_gpa: 2.2,
            poissons_ratio: 0.35,
            yield_strength_mpa: 40.0,
            density_kg_m3: 1040.0,
        }
    }

    /// Create from a standard preset
    pub fn from_preset(preset: MaterialPreset) -> Self {
        match preset {
            MaterialPreset::StructuralSteel => Self::structural_steel(),
            MaterialPreset::Aluminum6061 => Self::aluminum_6061(),
            MaterialPreset::TitaniumTi6Al4V => Self::titanium_ti6al4v(),
            MaterialPreset::PlaPlastic => Self::pla_plastic(),
            MaterialPreset::AbsPlastic => Self::abs_plastic(),
            MaterialPreset::Custom => Self::structural_steel(),
        }
    }

    /// Young's modulus in Pascals (N/m^2)
    pub fn youngs_modulus_pa(&self) -> f32 {
        self.youngs_modulus_gpa * 1.0e9
    }

    /// Shear Modulus G = E / (2 * (1 + nu)) in Pascals
    pub fn shear_modulus_pa(&self) -> f32 {
        self.youngs_modulus_pa() / (2.0 * (1.0 + self.poissons_ratio))
    }

    /// First Lamé Parameter lambda = (E * nu) / ((1 + nu) * (1 - 2*nu)) in Pascals
    pub fn lame_lambda_pa(&self) -> f32 {
        let e = self.youngs_modulus_pa();
        let nu = self.poissons_ratio;
        (e * nu) / ((1.0 + nu) * (1.0 - 2.0 * nu))
    }

    /// 6x6 Isotropic Elastic Constitutive Matrix [D] relating 3D strain to Cauchy stress
    /// Order: [sigma_xx, sigma_yy, sigma_zz, tau_xy, tau_yz, tau_zx]^T = [D] * [eps_xx, eps_yy, eps_zz, gamma_xy, gamma_yz, gamma_zx]^T
    pub fn constitutive_matrix_6x6(&self) -> [[f32; 6]; 6] {
        let e = self.youngs_modulus_pa();
        let nu = self.poissons_ratio;
        let factor = e / ((1.0 + nu) * (1.0 - 2.0 * nu));

        let c11 = factor * (1.0 - nu);
        let c12 = factor * nu;
        let g = self.shear_modulus_pa();

        [
            [c11, c12, c12, 0.0, 0.0, 0.0],
            [c12, c11, c12, 0.0, 0.0, 0.0],
            [c12, c12, c11, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, g,   0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, g,   0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, g  ],
        ]
    }
}
