/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::sync::Arc;
use std::thread;

use argon2_creds::{Config as ArgonConfig, ConfigBuilder as ArgonConfigBuilder, PasswordPolicy};

use crate::settings::Settings;

pub mod api;

/// App data
#[derive(Clone)]
pub struct Data {
    /// credential-procession policy
    pub creds: ArgonConfig,
    /// settings
    pub settings: Settings,
}

impl Data {
    /// Get credential-processing policy
    pub fn get_creds() -> ArgonConfig {
        ArgonConfigBuilder::default()
            .username_case_mapped(true)
            .profanity(true)
            .blacklist(true)
            .password_policy(PasswordPolicy::default())
            .build()
            .unwrap()
    }

    #[cfg(not(tarpaulin_include))]
    /// create new instance of app data
    pub fn new(settings: Option<Settings>) -> Arc<Self> {
        let settings = settings.unwrap_or_else(|| Settings::new().unwrap());
        let creds = Self::get_creds();
        let c = creds.clone();

        #[allow(unused_variables)]
        let init = thread::spawn(move || {
            log::info!("Initializing credential manager");
            c.init();
            log::info!("Initialized credential manager");
        });

        let data = Data { creds, settings };

        #[cfg(not(debug_assertions))]
        init.join().unwrap();

        Arc::new(data)
    }
}
