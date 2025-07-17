use crate::error::{BrowserError, BrowserResult};
use chromiumoxide::Page;
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{sleep, timeout};

pub struct HumanInteractor {
    page: Page,
}

impl HumanInteractor {
    pub fn new(page: Page) -> Self {
        Self { page }
    }

    pub async fn random_delay(&self, min_ms: u64, max_ms: u64) {
        let delay = rand::thread_rng().gen_range(min_ms..=max_ms);
        sleep(Duration::from_millis(delay)).await;
    }

    /// Enhanced stealth implementation with better error handling
    pub async fn apply_stealth(&self) -> BrowserResult<()> {
        let script = r#"
            (() => {
                try {
                    // Remove automation indicators
                    Object.defineProperty(navigator, 'webdriver', {
                        get: () => undefined,
                        configurable: true
                    });

                    // Enhanced chrome runtime mock
                    window.chrome = {
                        runtime: {
                            onConnect: undefined,
                            onMessage: undefined,
                            sendMessage: undefined,
                            getURL: () => 'chrome-extension://fake-extension-id/popup.html',
                            getManifest: () => ({
                                name: 'Chrome Extension',
                                version: '1.0.0'
                            })
                        },
                        storage: {
                            local: {
                                get: () => Promise.resolve({}),
                                set: () => Promise.resolve()
                            }
                        },
                        tabs: {
                            query: () => Promise.resolve([])
                        }
                    };

                    // Enhanced plugin mocking
                    Object.defineProperty(navigator, 'plugins', {
                        get: () => Object.setPrototypeOf([
                            {
                                name: 'PDF Viewer',
                                filename: 'internal-pdf-viewer',
                                description: 'Portable Document Format',
                                length: 2
                            },
                            {
                                name: 'Chrome PDF Plugin',
                                filename: 'internal-pdf-viewer',
                                description: 'Portable Document Format',
                                length: 2
                            },
                            {
                                name: 'Chromium PDF Plugin',
                                filename: 'internal-pdf-viewer',
                                description: 'Portable Document Format',
                                length: 2
                            }
                        ], Plugin.prototype),
                        configurable: true
                    });

                    // Enhanced language spoofing
                    Object.defineProperty(navigator, 'languages', {
                        get: () => ['en-US', 'en'],
                        configurable: true
                    });

                    Object.defineProperty(navigator, 'language', {
                        get: () => 'en-US',
                        configurable: true
                    });

                    // Mock realistic hardware specs
                    Object.defineProperty(navigator, 'hardwareConcurrency', {
                        get: () => 4,
                        configurable: true
                    });

                    Object.defineProperty(navigator, 'deviceMemory', {
                        get: () => 8,
                        configurable: true
                    });

                    // Enhanced permissions mocking
                    if (navigator.permissions && navigator.permissions.query) {
                        const originalQuery = navigator.permissions.query;
                        navigator.permissions.query = (parameters) => {
                            return Promise.resolve({
                                state: parameters.name === 'notifications' ? 'default' : 'granted',
                                onchange: null
                            });
                        };
                    }

                    // Enhanced connection mocking
                    Object.defineProperty(navigator, 'connection', {
                        get: () => ({
                            rtt: 50 + Math.random() * 50,
                            downlink: 10 + Math.random() * 5,
                            effectiveType: '4g',
                            saveData: false,
                            type: 'wifi'
                        }),
                        configurable: true
                    });

                    // Enhanced geolocation mocking
                    Object.defineProperty(navigator, 'geolocation', {
                        get: () => ({
                            getCurrentPosition: (success, error, options) => {
                                setTimeout(() => {
                                    if (error) error({ code: 1, message: 'User denied geolocation' });
                                }, 100);
                            },
                            watchPosition: () => Math.random() * 1000,
                            clearWatch: () => {}
                        }),
                        configurable: true
                    });

                    // Remove automation-specific properties
                    const propsToDelete = [
                        'domAutomation',
                        'domAutomationController',
                        '__nightmare',
                        '_phantom',
                        'callPhantom',
                        '__webdriver_script_fn',
                        '__webdriver_evaluate',
                        '__webdriver_unwrapped',
                        '__fxdriver_evaluate',
                        '__fxdriver_unwrapped',
                        '__driver_evaluate',
                        '__webdriver_script_func',
                        '__webdriver_script_function'
                    ];

                    propsToDelete.forEach(prop => {
                        if (window.hasOwnProperty(prop)) {
                            delete window[prop];
                        }
                    });

                    // Enhanced screen properties
                    Object.defineProperty(screen, 'availHeight', {
                        get: () => 1040,
                        configurable: true
                    });
                    Object.defineProperty(screen, 'availWidth', {
                        get: () => 1920,
                        configurable: true
                    });
                    Object.defineProperty(screen, 'height', {
                        get: () => 1080,
                        configurable: true
                    });
                    Object.defineProperty(screen, 'width', {
                        get: () => 1920,
                        configurable: true
                    });

                    // Mock touch support
                    Object.defineProperty(navigator, 'maxTouchPoints', {
                        get: () => 0,
                        configurable: true
                    });

                    // Enhanced battery API
                    Object.defineProperty(navigator, 'getBattery', {
                        get: () => () => Promise.resolve({
                            charging: true,
                            chargingTime: 0,
                            dischargingTime: Infinity,
                            level: 0.8 + Math.random() * 0.2,
                            addEventListener: () => {},
                            removeEventListener: () => {}
                        }),
                        configurable: true
                    });

                    // Mock User Agent Data
                    if (navigator.userAgentData) {
                        Object.defineProperty(navigator, 'userAgentData', {
                            get: () => ({
                                brands: [
                                    { brand: 'Google Chrome', version: '120' },
                                    { brand: 'Chromium', version: '120' },
                                    { brand: 'Not?A_Brand', version: '24' }
                                ],
                                mobile: false,
                                platform: 'Windows'
                            }),
                            configurable: true
                        });
                    }

                    // Enhanced iframe handling
                    const originalCreateElement = document.createElement;
                    document.createElement = function(tagName) {
                        const element = originalCreateElement.call(document, tagName);
                        if (tagName === 'iframe') {
                            try {
                                Object.defineProperty(element, 'contentWindow', {
                                    get: () => window,
                                    configurable: true
                                });
                            } catch (e) {}
                        }
                        return element;
                    };

                    // Mock clipboard API
                    if (!navigator.clipboard) {
                        Object.defineProperty(navigator, 'clipboard', {
                            get: () => ({
                                readText: () => Promise.resolve(''),
                                writeText: () => Promise.resolve()
                            }),
                            configurable: true
                        });
                    }

                    // Add realistic timing for method calls
                    const originalTimeout = window.setTimeout;
                    window.setTimeout = function(callback, delay) {
                        // Add slight randomization to timing
                        const variation = Math.random() * 2 - 1; // -1 to 1ms
                        return originalTimeout(callback, delay + variation);
                    };

                    return true;
                } catch (e) {
                    console.error('Stealth setup error:', e);
                    return false;
                }
            })();
        "#;

        let result = timeout(Duration::from_secs(10), self.page.evaluate(script))
            .await
            .map_err(|_| BrowserError::Script("Stealth setup timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    /// Enhanced session setup with realistic cookies and timing
    pub async fn setup_session(&self) -> BrowserResult<()> {
        let cookie_script = r#"
            (() => {
                try {
                    const now = new Date();
                    const futureDate = new Date(now.getTime() + 365 * 24 * 60 * 60 * 1000);

                    // Set realistic Google cookies
                    const cookies = [
                        `CONSENT=YES+cb.${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}-04-p0.en+FX+${Math.floor(Math.random() * 1000)}`,
                        `1P_JAR=${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}-${String(now.getHours()).padStart(2, '0')}`,
                        `NID=${Math.floor(Math.random() * 1000)}=${Array.from({length: 32}, () => Math.random().toString(36).charAt(0)).join('')}`,
                        `SOCS=CAESNQgDEitib3FfaWRlbnRpdHlmcm9udGVuZHVpc2VydmVyXzIwMjMxMjEzLjA4X3AwGgJlbg`,
                        `AEC=Ae3NU9M${Array.from({length: 40}, () => Math.random().toString(36).charAt(0)).join('')}`
                    ];

                    cookies.forEach(cookie => {
                        document.cookie = cookie + `; expires=${futureDate.toUTCString()}; path=/; domain=.google.com; SameSite=Lax`;
                    });

                    // Set additional session storage
                    if (window.sessionStorage) {
                        window.sessionStorage.setItem('google_session', JSON.stringify({
                            timestamp: now.getTime(),
                            visits: Math.floor(Math.random() * 10) + 1
                        }));
                    }

                    return true;
                } catch (e) {
                    return false;
                }
            })();
        "#;

        let result = timeout(Duration::from_secs(5), self.page.evaluate(cookie_script))
            .await
            .map_err(|_| BrowserError::Script("Session setup timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    /// Enhanced browsing history simulation
    pub async fn simulate_browsing_history(&self) -> BrowserResult<()> {
        let history_script = r#"
            (() => {
                try {
                    const searches = ['weather', 'news', 'recipes', 'travel', 'shopping'];
                    const randomSearch = searches[Math.floor(Math.random() * searches.length)];

                    // Simulate some browsing history
                    window.history.pushState({page: 1}, '', `/?q=${randomSearch}`);
                    window.history.pushState({page: 2}, '', `/search?q=${randomSearch}`);

                    // Go back to simulate normal browsing
                    setTimeout(() => {
                        window.history.back();
                    }, 100);

                    return true;
                } catch (e) {
                    return false;
                }
            })();
        "#;

        let result = timeout(Duration::from_secs(5), self.page.evaluate(history_script))
            .await
            .map_err(|_| BrowserError::Script("History simulation timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    /// Enhanced human reading behavior with more realistic patterns
    pub async fn simulate_human_reading(&self) -> BrowserResult<()> {
        let reading_patterns = [
            (300, 600),   // Quick scan
            (800, 1200),  // Normal reading
            (1500, 2500), // Deep reading
            (150, 300),   // Skip
            (2000, 4000), // Careful reading
        ];

        let pattern = reading_patterns[rand::thread_rng().gen_range(0..reading_patterns.len())];
        self.random_delay(pattern.0, pattern.1).await;

        // Simulate realistic mouse movements
        let movements = rand::thread_rng().gen_range(3..7);
        for _ in 0..movements {
            let x = rand::thread_rng().gen_range(200.0..1000.0);
            let y = rand::thread_rng().gen_range(300.0..700.0);
            let steps = rand::thread_rng().gen_range(8..15);

            if let Err(_) = self.move_mouse(x, y, steps).await {
                // Continue on mouse movement errors
            }

            self.random_delay(200, 800).await;
        }

        // Simulate occasional scrolling while reading
        if rand::thread_rng().gen_bool(0.6) {
            let scroll_amount = rand::thread_rng().gen_range(50..200);
            let _ = self.scroll_human(scroll_amount, 3).await;
        }

        Ok(())
    }

    /// Enhanced hesitation simulation
    pub async fn simulate_hesitation(&self) -> BrowserResult<()> {
        let hesitation_time = rand::thread_rng().gen_range(1500..4000);
        self.random_delay(hesitation_time, hesitation_time + 1000)
            .await;

        // Simulate various hesitation behaviors
        let behavior = rand::thread_rng().gen_range(0..4);
        match behavior {
            0 => {
                // Small mouse movements
                let x = rand::thread_rng().gen_range(300.0..700.0);
                let y = rand::thread_rng().gen_range(400.0..600.0);
                let _ = self.move_mouse(x, y, 8).await;
            }
            1 => {
                // Scroll up a bit then down
                let _ = self.scroll_human(-50, 2).await;
                self.random_delay(500, 1000).await;
                let _ = self.scroll_human(50, 2).await;
            }
            2 => {
                // Focus and unfocus search box
                let _ = self.focus_blur("textarea[name=\"q\"]").await;
            }
            _ => {
                // Just wait
            }
        }

        Ok(())
    }

    /// Enhanced tab switching simulation
    pub async fn simulate_tab_switching(&self) -> BrowserResult<()> {
        let script = r#"
            (() => {
                try {
                    // Simulate realistic tab switching
                    const blurEvent = new Event('blur', { bubbles: true });
                    const focusEvent = new Event('focus', { bubbles: true });

                    // Simulate losing focus
                    window.dispatchEvent(blurEvent);
                    document.dispatchEvent(blurEvent);

                    // Simulate regaining focus after random time
                    setTimeout(() => {
                        window.dispatchEvent(focusEvent);
                        document.dispatchEvent(focusEvent);
                    }, 1000 + Math.random() * 3000);

                    return true;
                } catch (e) {
                    return false;
                }
            })();
        "#;

        let result = timeout(Duration::from_secs(5), self.page.evaluate(script))
            .await
            .map_err(|_| BrowserError::Script("Tab switching timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        self.random_delay(1000, 4000).await;
        Ok(())
    }

    /// Enhanced realistic search patterns
    pub async fn search_with_realistic_pattern(&self, query: &str) -> BrowserResult<()> {
        // Sometimes start typing then pause and think
        if rand::thread_rng().gen_bool(0.4) {
            let partial_length = rand::thread_rng().gen_range(1..query.len().min(4));
            let partial = &query[..partial_length];

            if let Err(_) = self.type_human("textarea[name=\"q\"]", partial).await {
                // Try alternative selector
                let _ = self.type_human("input[name=\"q\"]", partial).await;
            }

            self.random_delay(800, 2000).await;

            // Clear and start over
            self.clear_search_box().await?;
            self.random_delay(300, 800).await;
        }

        // Normal typing with enhanced human patterns
        self.type_in_google_search(query).await?;

        // Random hesitation before submitting
        if rand::thread_rng().gen_bool(0.3) {
            self.simulate_hesitation().await?;
        }

        Ok(())
    }

    /// Enhanced search box clearing
    pub async fn clear_search_box(&self) -> BrowserResult<()> {
        let script = r#"
            (() => {
                try {
                    const selectors = ['textarea[name="q"]', 'input[name="q"]'];
                    for (const selector of selectors) {
                        const element = document.querySelector(selector);
                        if (element) {
                            // Simulate realistic clearing
                            element.focus();
                            element.select();

                            // Simulate Ctrl+A then delete
                            element.dispatchEvent(new KeyboardEvent('keydown', {
                                key: 'a',
                                ctrlKey: true,
                                bubbles: true
                            }));

                            setTimeout(() => {
                                element.value = '';
                                element.dispatchEvent(new Event('input', { bubbles: true }));
                            }, 50);

                            return true;
                        }
                    }
                    return false;
                } catch (e) {
                    return false;
                }
            })();
        "#;

        let result = timeout(Duration::from_secs(5), self.page.evaluate(script))
            .await
            .map_err(|_| BrowserError::Script("Clear search box timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    /// Enhanced CAPTCHA detection
    pub async fn check_for_captcha(&self) -> BrowserResult<bool> {
        let script = r#"
            (() => {
                try {
                    const captchaSelectors = [
                        '.g-recaptcha',
                        '#captcha-form',
                        '.captcha-container',
                        '[data-recaptcha]',
                        'iframe[src*="recaptcha"]'
                    ];

                    // Check for CAPTCHA elements
                    for (const selector of captchaSelectors) {
                        if (document.querySelector(selector)) {
                            return true;
                        }
                    }

                    // Check page content for bot detection
                    const bodyText = document.body.innerText || document.body.textContent || '';
                    const titleText = document.title || '';

                    const botKeywords = [
                        'unusual traffic',
                        'verify you are human',
                        'robot',
                        'automated',
                        'captcha',
                        'verify that you are not a robot',
                        'please complete the security check',
                        'blocked'
                    ];

                    const combinedText = (bodyText + ' ' + titleText).toLowerCase();

                    return botKeywords.some(keyword => combinedText.includes(keyword));
                } catch (e) {
                    return false;
                }
            })();
        "#;

        let result = timeout(Duration::from_secs(5), self.page.evaluate(script))
            .await
            .map_err(|_| BrowserError::Script("CAPTCHA check timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let has_captcha: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(has_captcha)
    }

    /// Enhanced mouse movement with more realistic curves
    pub async fn move_mouse(&self, x: f64, y: f64, steps: usize) -> BrowserResult<()> {
        let current_pos = self.get_mouse_position().await.unwrap_or((0.0, 0.0));
        let start_x = current_pos.0;
        let start_y = current_pos.1;

        for i in 0..steps {
            let progress = i as f64 / steps as f64;

            // Enhanced curve with multiple wave components
            let curve_factor = 1.0 - (progress - 0.5).powi(2) * 4.0;
            let wave1 = (progress * std::f64::consts::PI * 2.0).sin() * 3.0;
            let wave2 = (progress * std::f64::consts::PI * 4.0).sin() * 1.5;
            let noise = rand::thread_rng().gen_range(-1.5..1.5);

            let current_x = start_x + (x - start_x) * progress + wave1 + noise;
            let current_y = start_y + (y - start_y) * progress + curve_factor * 8.0 + wave2 + noise;

            let script = format!(
                r#"
                (() => {{
                    try {{
                        window.mouseX = {};
                        window.mouseY = {};

                        const event = new MouseEvent('mousemove', {{
                            clientX: {},
                            clientY: {},
                            bubbles: true,
                            cancelable: true
                        }});

                        document.dispatchEvent(event);
                        return true;
                    }} catch (e) {{
                        return false;
                    }}
                }})();
            "#,
                current_x, current_y, current_x, current_y
            );

            let result = timeout(
                Duration::from_millis(100),
                self.page.evaluate(script.as_str()),
            )
            .await
            .map_err(|_| BrowserError::Script("Mouse move timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

            let _success: bool = result
                .into_value()
                .map_err(|e| BrowserError::Script(e.to_string()))?;

            // Variable delay between movements
            let delay = rand::thread_rng().gen_range(8..25);
            self.random_delay(delay, delay + 10).await;
        }

        Ok(())
    }

    /// Enhanced typing with more realistic human patterns
    pub async fn type_human(&self, selector: &str, text: &str) -> BrowserResult<()> {
        let script = format!(
            r#"
            (() => {{
                try {{
                    const element = document.querySelector('{}');
                    if (!element) return false;

                    // Simulate realistic focus
                    element.focus();
                    element.click();
                    element.value = '';

                    return true;
                }} catch (e) {{
                    return false;
                }}
            }})();
        "#,
            selector
        );

        let result = timeout(Duration::from_secs(5), self.page.evaluate(script.as_str()))
            .await
            .map_err(|_| BrowserError::Script("Element focus timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let found: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        if !found {
            return Err(BrowserError::ElementNotFound(selector.to_string()));
        }

        // Enhanced typing with more realistic patterns
        for (i, ch) in text.chars().enumerate() {
            // Random typos (2% chance)
            if i > 0 && rand::thread_rng().gen_bool(0.02) {
                let wrong_chars = ['a', 'e', 'i', 'o', 'u', 'n', 't', 's', 'r'];
                let wrong_char = wrong_chars[rand::thread_rng().gen_range(0..wrong_chars.len())];

                self.type_character(selector, wrong_char).await?;
                self.random_delay(150, 300).await;
                self.send_key(selector, "Backspace").await?;
                self.random_delay(100, 200).await;
            }

            // Type the correct character
            self.type_character(selector, ch).await?;

            // Variable typing speed
            let base_delay = match ch {
                ' ' => rand::thread_rng().gen_range(100..200),
                '.' | ',' | '!' | '?' => rand::thread_rng().gen_range(150..250),
                _ => rand::thread_rng().gen_range(80..180),
            };

            self.random_delay(base_delay, base_delay + 50).await;

            // Random pauses (thinking)
            if rand::thread_rng().gen_bool(0.08) {
                self.random_delay(300, 1000).await;
            }
        }

        Ok(())
    }

    /// Enhanced character typing with better event simulation
    async fn type_character(&self, selector: &str, ch: char) -> BrowserResult<()> {
        let script = format!(
            r#"
            (() => {{
                try {{
                    const element = document.querySelector('{}');
                    if (!element) return false;

                    // Simulate realistic key events
                    const keyCode = '{}';

                    // Dispatch keydown
                    element.dispatchEvent(new KeyboardEvent('keydown', {{
                        key: keyCode,
                        code: keyCode,
                        bubbles: true,
                        cancelable: true
                    }}));

                    // Update value
                    element.value += '{}';

                    // Dispatch input event
                    element.dispatchEvent(new Event('input', {{
                        bubbles: true,
                        cancelable: true
                    }}));

                    // Dispatch keyup
                    element.dispatchEvent(new KeyboardEvent('keyup', {{
                        key: keyCode,
                        code: keyCode,
                        bubbles: true,
                        cancelable: true
                    }}));

                    return true;
                }} catch (e) {{
                    return false;
                }}
            }})();
        "#,
            selector, ch, ch
        );

        let result = timeout(
            Duration::from_millis(500),
            self.page.evaluate(script.as_str()),
        )
        .await
        .map_err(|_| BrowserError::Script("Character type timeout".to_string()))?
        .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    /// Enhanced key sending with proper event simulation
    async fn send_key(&self, selector: &str, key: &str) -> BrowserResult<()> {
        let script = format!(
            r#"
            (() => {{
                try {{
                    const element = document.querySelector('{}');
                    if (!element) return false;

                    const keyEvent = new KeyboardEvent('keydown', {{
                        key: '{}',
                        code: '{}',
                        bubbles: true,
                        cancelable: true
                    }});

                    element.dispatchEvent(keyEvent);

                    if ('{}' === 'Backspace') {{
                        element.value = element.value.slice(0, -1);
                        element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    }}

                    return true;
                }} catch (e) {{
                    return false;
                }}
            }})();
        "#,
            selector, key, key, key
        );

        let result = timeout(
            Duration::from_millis(500),
            self.page.evaluate(script.as_str()),
        )
        .await
        .map_err(|_| BrowserError::Script("Key send timeout".to_string()))?
        .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    /// Enhanced scrolling with more realistic behavior
    pub async fn scroll_human(&self, amount: i32, steps: usize) -> BrowserResult<()> {
        let step_size = amount / steps as i32;
        let variation = amount as f64 * 0.1; // 10% variation

        for i in 0..steps {
            let progress = i as f64 / steps as f64;
            let ease = 1.0 - (1.0 - progress) * (1.0 - progress); // Ease out
            let varied_step = (step_size as f64 * ease
                + rand::thread_rng().gen_range(-variation..variation))
                as i32;

            let script = format!(
                r#"
                (() => {{
                    try {{
                        window.scrollBy({{
                            top: {},
                            left: 0,
                            behavior: 'smooth'
                        }});
                        return true;
                    }} catch (e) {{
                        return false;
                    }}
                }})();
            "#,
                varied_step
            );

            let result = timeout(
                Duration::from_millis(500),
                self.page.evaluate(script.as_str()),
            )
            .await
            .map_err(|_| BrowserError::Script("Scroll timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

            let _success: bool = result
                .into_value()
                .map_err(|e| BrowserError::Script(e.to_string()))?;

            // Variable delay between scroll steps
            let delay = rand::thread_rng().gen_range(100..400);
            self.random_delay(delay, delay + 100).await;
        }

        Ok(())
    }

    // Continue with remaining methods...
    async fn get_mouse_position(&self) -> BrowserResult<(f64, f64)> {
        let script = r#"
            (() => {
                try {
                    return { x: window.mouseX || window.innerWidth / 2, y: window.mouseY || window.innerHeight / 2 };
                } catch (e) {
                    return { x: 683, y: 384 };
                }
            })();
        "#;

        let result = timeout(Duration::from_millis(1000), self.page.evaluate(script))
            .await
            .map_err(|_| BrowserError::Script("Mouse position timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let pos: serde_json::Value = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let x = pos["x"].as_f64().unwrap_or(683.0);
        let y = pos["y"].as_f64().unwrap_or(384.0);

        Ok((x, y))
    }

    pub async fn focus_blur(&self, selector: &str) -> BrowserResult<()> {
        let script = format!(
            r#"
            (() => {{
                try {{
                    const element = document.querySelector('{}');
                    if (!element) return false;

                    element.focus();
                    setTimeout(() => {{
                        element.blur();
                    }}, 150);

                    return true;
                }} catch (e) {{
                    return false;
                }}
            }})();
        "#,
            selector
        );

        let result = timeout(Duration::from_secs(2), self.page.evaluate(script.as_str()))
            .await
            .map_err(|_| BrowserError::Script("Focus blur timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        if !success {
            return Err(BrowserError::ElementNotFound(selector.to_string()));
        }

        self.random_delay(200, 400).await;
        Ok(())
    }

    pub async fn click_human(&self, selector: &str) -> BrowserResult<()> {
        // Get element position with timeout
        let position = timeout(Duration::from_secs(5), self.get_element_position(selector))
            .await
            .map_err(|_| BrowserError::Script("Element position timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        // Move mouse to element
        self.move_mouse(position.0, position.1, 12).await?;
        self.random_delay(100, 200).await;

        let script = format!(
            r#"
            (() => {{
                try {{
                    const element = document.querySelector('{}');
                    if (!element) return false;

                    const rect = element.getBoundingClientRect();
                    const x = rect.left + rect.width / 2;
                    const y = rect.top + rect.height / 2;

                    // Simulate complete click sequence
                    element.dispatchEvent(new MouseEvent('mousedown', {{
                        clientX: x,
                        clientY: y,
                        bubbles: true,
                        cancelable: true
                    }}));

                    setTimeout(() => {{
                        element.dispatchEvent(new MouseEvent('mouseup', {{
                            clientX: x,
                            clientY: y,
                            bubbles: true,
                            cancelable: true
                        }}));

                        element.click();
                    }}, 50 + Math.random() * 50);

                    return true;
                }} catch (e) {{
                    return false;
                }}
            }})();
        "#,
            selector
        );

        let result = timeout(Duration::from_secs(5), self.page.evaluate(script.as_str()))
            .await
            .map_err(|_| BrowserError::Script("Click timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        if !success {
            return Err(BrowserError::ElementNotFound(selector.to_string()));
        }

        Ok(())
    }

    async fn get_element_position(&self, selector: &str) -> BrowserResult<(f64, f64)> {
        let script = format!(
            r#"
            (() => {{
                try {{
                    const element = document.querySelector('{}');
                    if (!element) return null;

                    const rect = element.getBoundingClientRect();

                    // Add small random offset to make clicking more human-like
                    const offsetX = (Math.random() - 0.5) * Math.min(rect.width * 0.3, 10);
                    const offsetY = (Math.random() - 0.5) * Math.min(rect.height * 0.3, 10);

                    return {{
                        x: rect.left + rect.width / 2 + offsetX,
                        y: rect.top + rect.height / 2 + offsetY
                    }};
                }} catch (e) {{
                    return null;
                }}
            }})();
        "#,
            selector
        );

        let result = timeout(Duration::from_secs(3), self.page.evaluate(script.as_str()))
            .await
            .map_err(|_| BrowserError::Script("Element position timeout".to_string()))?
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let pos: serde_json::Value = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        if pos.is_null() {
            return Err(BrowserError::ElementNotFound(selector.to_string()));
        }

        let x = pos["x"].as_f64().unwrap_or(0.0);
        let y = pos["y"].as_f64().unwrap_or(0.0);

        Ok((x, y))
    }

    pub async fn reading_delay(&self, text: &str) {
        let words = text.split_whitespace().count();
        let base_time = (words as f64 * 60.0) + 200.0; // Base reading time
        let variation = base_time * 0.3; // 30% variation

        let min_delay = (base_time - variation) as u64;
        let max_delay = (base_time + variation) as u64;

        self.random_delay(min_delay, max_delay).await;
    }

    pub async fn idle_time(&self) {
        let idle_patterns = [
            (800, 1500),  // Quick idle
            (1500, 3000), // Normal idle
            (3000, 5000), // Long idle
        ];

        let pattern = idle_patterns[rand::thread_rng().gen_range(0..idle_patterns.len())];
        self.random_delay(pattern.0, pattern.1).await;
    }

    pub async fn type_in_google_search(&self, text: &str) -> BrowserResult<()> {
        let selectors = [
            "textarea[name=\"q\"]",
            "input[name=\"q\"]",
            "input[aria-label=\"Search\"]",
            "textarea[aria-label=\"Search\"]",
            "input[type=\"search\"]",
        ];

        for selector in &selectors {
            let script = format!(
                r#"
                (() => {{
                    try {{
                        const element = document.querySelector('{}');
                        if (!element) return false;

                        // Simulate realistic interaction
                        element.focus();
                        element.click();
                        element.value = '';

                        return true;
                    }} catch (e) {{
                        return false;
                    }}
                }})();
            "#,
                selector
            );

            let result = timeout(Duration::from_secs(3), self.page.evaluate(script.as_str()))
                .await
                .map_err(|_| BrowserError::Script("Search box focus timeout".to_string()))?
                .map_err(|e| BrowserError::Script(e.to_string()))?;

            let found: bool = result
                .into_value()
                .map_err(|e| BrowserError::Script(e.to_string()))?;

            if found {
                self.type_human(selector, text).await?;
                return Ok(());
            }
        }

        Err(BrowserError::ElementNotFound(
            "Google search box".to_string(),
        ))
    }

    pub async fn submit_search(&self) -> BrowserResult<()> {
        // Method 1: Try Enter key on search input
        let search_selectors = ["textarea[name=\"q\"]", "input[name=\"q\"]"];
        for selector in &search_selectors {
            let script = format!(
                r#"
                (() => {{
                    try {{
                        const element = document.querySelector('{}');
                        if (!element) return false;

                        element.focus();

                        // Simulate realistic Enter key press
                        const event = new KeyboardEvent('keydown', {{
                            key: 'Enter',
                            code: 'Enter',
                            keyCode: 13,
                            which: 13,
                            bubbles: true,
                            cancelable: true
                        }});

                        element.dispatchEvent(event);

                        // Also try keypress for older browsers
                        const keypressEvent = new KeyboardEvent('keypress', {{
                            key: 'Enter',
                            code: 'Enter',
                            keyCode: 13,
                            which: 13,
                            bubbles: true,
                            cancelable: true
                        }});

                        element.dispatchEvent(keypressEvent);

                        return true;
                    }} catch (e) {{
                        return false;
                    }}
                }})();
            "#,
                selector
            );

            let result = timeout(Duration::from_secs(5), self.page.evaluate(script.as_str()))
                .await
                .map_err(|_| BrowserError::Script("Search submit timeout".to_string()))?
                .map_err(|e| BrowserError::Script(e.to_string()))?;

            let success: bool = result
                .into_value()
                .map_err(|e| BrowserError::Script(e.to_string()))?;

            if success {
                return Ok(());
            }
        }

        // Method 2: Try clicking search button
        let button_selectors = [
            "input[name=\"btnK\"]",
            "button[aria-label=\"Google Search\"]",
            "input[value=\"Google Search\"]",
            "button[type=\"submit\"]",
        ];

        for selector in &button_selectors {
            let script = format!(
                r#"
                (() => {{
                    try {{
                        const element = document.querySelector('{}');
                        if (!element) return false;

                        // Check if element is visible
                        const rect = element.getBoundingClientRect();
                        if (rect.width > 0 && rect.height > 0 &&
                            getComputedStyle(element).display !== 'none') {{

                            element.click();
                            return true;
                        }}
                        return false;
                    }} catch (e) {{
                        return false;
                    }}
                }})();
            "#,
                selector
            );

            let result = timeout(Duration::from_secs(3), self.page.evaluate(script.as_str()))
                .await
                .map_err(|_| BrowserError::Script("Button click timeout".to_string()))?
                .map_err(|e| BrowserError::Script(e.to_string()))?;

            let success: bool = result
                .into_value()
                .map_err(|e| BrowserError::Script(e.to_string()))?;

            if success {
                return Ok(());
            }
        }

        Err(BrowserError::ElementNotFound(
            "Search submission failed".to_string(),
        ))
    }

    pub async fn wait_for_navigation(&self) -> BrowserResult<()> {
        match timeout(Duration::from_secs(15), self.page.wait_for_navigation()).await {
            Ok(Ok(_page)) => Ok(()),
            Ok(Err(_)) => {
                // Navigation failed, wait and continue
                self.random_delay(2000, 4000).await;
                Ok(())
            }
            Err(_) => {
                // Timeout, wait and continue
                self.random_delay(3000, 5000).await;
                Ok(())
            }
        }
    }

    pub async fn is_on_results_page(&self) -> BrowserResult<bool> {
        let script = r#"
            (() => {
                try {
                    const url = window.location.href;
                    const hasSearchInUrl = url.includes('/search') || url.includes('?q=');

                    const resultSelectors = [
                        '#search',
                        '.g',
                        '.MjjYud',
                        '.tF2Cxc',
                        '#result-stats',
                        '.s75CSd', // New Google result selector
                        '.kvH3mc'  // Another result selector
                    ];

                    const hasResultElements = resultSelectors.some(selector =>
                        document.querySelector(selector) !== null
                    );

                    return hasSearchInUrl || hasResultElements;
                } catch (e) {
                    return false;
                }
            })();
        "#;

        match timeout(Duration::from_secs(5), self.page.evaluate(script)).await {
            Ok(Ok(result)) => {
                let is_results: bool = result
                    .into_value()
                    .map_err(|e| BrowserError::Script(e.to_string()))?;
                Ok(is_results)
            }
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    pub async fn extract_search_results(&self) -> BrowserResult<Vec<HashMap<String, String>>> {
        let script = r#"
            (() => {
                try {
                    const results = [];
                    const resultSelectors = [
                        '.g',
                        '.MjjYud',
                        '.tF2Cxc',
                        '.rc',
                        '.s75CSd',
                        '.kvH3mc'
                    ];

                    let resultElements = [];

                    for (const selector of resultSelectors) {
                        const elements = document.querySelectorAll(selector);
                        if (elements.length > 0) {
                            resultElements = Array.from(elements);
                            break;
                        }
                    }

                    resultElements.forEach(element => {
                        try {
                            const titleSelectors = ['h3', 'h1', 'h2', '[role="heading"]'];
                            let titleElement = null;

                            for (const selector of titleSelectors) {
                                titleElement = element.querySelector(selector);
                                if (titleElement) break;
                            }

                            const linkElement = element.querySelector('a[href]');

                            const snippetSelectors = [
                                '.VwiC3b',
                                '.s3v9rd',
                                '.st',
                                '.IsZvec',
                                '.aCOpRe'
                            ];

                            let snippetElement = null;
                            for (const selector of snippetSelectors) {
                                snippetElement = element.querySelector(selector);
                                if (snippetElement) break;
                            }

                            if (titleElement && linkElement) {
                                const title = titleElement.textContent || titleElement.innerText || '';
                                const link = linkElement.href || '';
                                const snippet = snippetElement ?
                                    (snippetElement.textContent || snippetElement.innerText || '') : '';

                                if (title.trim() && link.trim() &&
                                    !link.includes('javascript:') &&
                                    !link.includes('google.com/search')) {

                                    results.push({
                                        title: title.trim(),
                                        link: link.trim(),
                                        snippet: snippet.trim()
                                    });
                                }
                            }
                        } catch (e) {
                            // Skip this element
                        }
                    });

                    return results;
                } catch (e) {
                    return [];
                }
            })();
        "#;

        match timeout(Duration::from_secs(10), self.page.evaluate(script)).await {
            Ok(Ok(result)) => {
                let results: Vec<HashMap<String, String>> = result
                    .into_value()
                    .map_err(|e| BrowserError::Script(e.to_string()))?;
                Ok(results)
            }
            Ok(Err(_)) => Ok(Vec::new()),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn wait_for_results(&self, timeout_seconds: u64) -> BrowserResult<()> {
        let start_time = std::time::Instant::now();

        while start_time.elapsed().as_secs() < timeout_seconds {
            match self.is_on_results_page().await {
                Ok(true) => return Ok(()),
                Ok(false) => {
                    self.random_delay(800, 1500).await;
                }
                Err(_) => {
                    self.random_delay(1000, 2000).await;
                }
            }
        }

        Err(BrowserError::Script(
            "Timeout waiting for search results".to_string(),
        ))
    }

    pub async fn get_current_url(&self) -> BrowserResult<String> {
        let script = r#"
            (() => {
                try {
                    return window.location.href;
                } catch (e) {
                    return '';
                }
            })();
        "#;

        match timeout(Duration::from_secs(3), self.page.evaluate(script)).await {
            Ok(Ok(result)) => {
                let url: String = result
                    .into_value()
                    .map_err(|e| BrowserError::Script(e.to_string()))?;
                Ok(url)
            }
            Ok(Err(_)) => Ok(String::new()),
            Err(_) => Ok(String::new()),
        }
    }

    pub async fn is_page_loaded(&self) -> BrowserResult<bool> {
        let script = r#"
            (() => {
                try {
                    return document.readyState === 'complete' &&
                           !document.querySelector('.loading') &&
                           !document.querySelector('[data-loading]');
                } catch (e) {
                    return false;
                }
            })();
        "#;

        match timeout(Duration::from_secs(3), self.page.evaluate(script)).await {
            Ok(Ok(result)) => {
                let loaded: bool = result
                    .into_value()
                    .map_err(|e| BrowserError::Script(e.to_string()))?;
                Ok(loaded)
            }
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    pub async fn wait_for_page_load(&self, timeout_seconds: u64) -> BrowserResult<()> {
        let start_time = std::time::Instant::now();

        while start_time.elapsed().as_secs() < timeout_seconds {
            match self.is_page_loaded().await {
                Ok(true) => return Ok(()),
                Ok(false) => {
                    self.random_delay(300, 800).await;
                }
                Err(_) => {
                    self.random_delay(500, 1000).await;
                }
            }
        }

        Err(BrowserError::Script(
            "Timeout waiting for page to load".to_string(),
        ))
    }
}
