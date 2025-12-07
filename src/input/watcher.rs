//! Configuration file watcher for hot-reload support
//!
//! This module provides the `ConfigWatcher` that monitors a configuration file
//! for changes and triggers reload events. It uses the `notify` crate with
//! debouncing to handle file system events efficiently.
//!
//! # Features
//!
//! - **Debounced Events**: Prevents multiple reloads for a single save operation
//! - **Cross-Platform**: Works on Linux, macOS, and Windows
//! - **Non-Blocking**: Check for changes without blocking the main loop
//! - **Error Tolerant**: Watcher errors don't crash the application
//!
//! # Examples
//!
//! ```no_run
//! use termide::input::watcher::ConfigWatcher;
//! use std::path::Path;
//!
//! let config_path = Path::new("~/.config/termide/config.toml");
//! let mut watcher = ConfigWatcher::new(config_path)?;
//!
//! // In main event loop
//! if watcher.check_for_changes() {
//!     println!("Config file changed, reloading...");
//!     // Call reload_user_keybindings()
//! }
//! # Ok::<(), notify::Error>(())
//! ```

use notify::Watcher;
use notify_debouncer_mini::new_debouncer;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Config file watcher for hot-reload support
///
/// Monitors the configuration file for changes and provides a mechanism to check
/// if the file has been modified since the last check. Uses a debounced file watcher
/// to avoid triggering multiple reloads for a single save operation.
///
/// # Implementation Details
///
/// - Uses `notify` for cross-platform file system events
/// - Debounces events with 500ms timeout to handle rapid saves
/// - Non-blocking checks via `try_recv()` on event channel
/// - Watches the file directly, not the parent directory
///
/// # Lifetime
///
/// The watcher remains active as long as the `ConfigWatcher` struct is alive.
/// Dropping the struct stops file watching automatically.
///
/// # Examples
///
/// ```no_run
/// use termide::input::watcher::ConfigWatcher;
/// use termide::input::config::get_config_path;
///
/// if let Some(config_path) = get_config_path() {
///     match ConfigWatcher::new(&config_path) {
///         Ok(mut watcher) => {
///             // In main loop
///             loop {
///                 if watcher.check_for_changes() {
///                     println!("Config file changed, reloading...");
///                     // Reload logic here
///                 }
///                 // Other event handling...
///             }
///         }
///         Err(e) => {
///             eprintln!("Failed to start config watcher: {}", e);
///             eprintln!("Hot reload disabled, restart editor to apply config changes");
///         }
///     }
/// }
/// ```
pub struct ConfigWatcher {
    /// Debouncer must be kept alive to continue watching
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
    /// Channel receiver for debounced file system events
    receiver: Receiver<Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>>,
}

impl ConfigWatcher {
    /// Creates a new config file watcher
    ///
    /// Sets up a debounced file watcher that monitors the specified config file
    /// for modification, creation, and deletion events. Events are debounced with
    /// a 500ms timeout to avoid rapid-fire triggers when editors save files
    /// multiple times in quick succession.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file to watch
    ///
    /// # Returns
    ///
    /// - `Ok(ConfigWatcher)` - Successfully started watching the file
    /// - `Err(notify::Error)` - Failed to start file watcher (file doesn't exist, permission denied, etc.)
    ///
    /// # Errors
    ///
    /// This function can fail if:
    /// - The file doesn't exist (config file must exist before watching)
    /// - Insufficient permissions to watch the file or directory
    /// - Platform-specific file watcher initialization fails
    ///
    /// If watcher creation fails, the application should continue without hot reload.
    ///
    /// # Platform Behavior
    ///
    /// - **Linux**: Uses `inotify` for efficient event notification
    /// - **macOS**: Uses `FSEvents` API
    /// - **Windows**: Uses `ReadDirectoryChangesW`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use termide::input::watcher::ConfigWatcher;
    /// use std::path::Path;
    ///
    /// let config_path = Path::new("~/.config/termide/config.toml");
    ///
    /// match ConfigWatcher::new(config_path) {
    ///     Ok(watcher) => {
    ///         println!("✓ Config watcher started");
    ///         // Use watcher in event loop
    ///     }
    ///     Err(e) => {
    ///         eprintln!("⚠ Could not start config watcher: {}", e);
    ///         eprintln!("  Hot reload disabled");
    ///         // Continue without hot reload
    ///     }
    /// }
    /// # Ok::<(), notify::Error>(())
    /// ```
    ///
    /// # Debouncing Explained
    ///
    /// When you save a file in most editors, the OS may generate multiple events:
    /// 1. Write to temporary file
    /// 2. Rename temporary file to target
    /// 3. Update file metadata
    ///
    /// Without debouncing, this would trigger 3 reloads. The debouncer waits 500ms
    /// after the first event, collecting any additional events in that window, then
    /// delivers them as a single batch.
    pub fn new(config_path: &Path) -> Result<Self, notify::Error> {
        let (tx, rx) = channel();

        // Create debouncer with 500ms timeout
        // This prevents rapid-fire reloads when editors save multiple times
        let mut debouncer = new_debouncer(Duration::from_millis(500), tx)?;

        // Watch the config file
        // Use NonRecursive since we're watching a single file, not a directory tree
        debouncer
            .watcher()
            .watch(config_path, notify::RecursiveMode::NonRecursive)?;

        Ok(Self {
            _debouncer: debouncer,
            receiver: rx,
        })
    }

    /// Checks if the config file has changed since the last check
    ///
    /// This is a **non-blocking** call that drains all pending events from the watcher.
    /// Returns `true` if any modification events were detected. This should be called
    /// periodically from the main event loop to detect config changes.
    ///
    /// # Returns
    ///
    /// - `true` if the file was modified, created, or deleted
    /// - `false` if no changes detected or if there were watcher errors
    ///
    /// # Behavior
    ///
    /// - Drains **all** pending events from the channel
    /// - Logs errors to stderr but doesn't propagate them
    /// - Returns `true` if **any** event indicates a change
    /// - Safe to call frequently (does not block)
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```no_run
    /// use termide::input::watcher::ConfigWatcher;
    /// use std::path::Path;
    ///
    /// let config_path = Path::new("~/.config/termide/config.toml");
    /// let mut watcher = ConfigWatcher::new(config_path)?;
    ///
    /// // In event loop
    /// loop {
    ///     if watcher.check_for_changes() {
    ///         println!("Config changed!");
    ///         // Trigger reload
    ///     }
    ///
    ///     // Handle other events...
    ///     std::thread::sleep(std::time::Duration::from_millis(100));
    /// }
    /// # Ok::<(), notify::Error>(())
    /// ```
    ///
    /// ## With Reload Logic
    ///
    /// ```no_run
    /// use termide::input::watcher::ConfigWatcher;
    /// use termide::input::config::{reload_user_keybindings, get_config_path};
    /// use termide::input::registry::KeyBindingRegistry;
    /// use std::time::Duration;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    /// let config_path = get_config_path().unwrap();
    /// let mut watcher = ConfigWatcher::new(&config_path)?;
    ///
    /// loop {
    ///     if watcher.check_for_changes() {
    ///         match reload_user_keybindings(&mut registry, &config_path) {
    ///             Ok((removed, loaded)) => {
    ///                 println!("✓ Config reloaded: {} bindings", loaded);
    ///             }
    ///             Err(e) => {
    ///                 eprintln!("⚠ Config reload failed: {}", e);
    ///             }
    ///         }
    ///     }
    ///
    ///     // Handle key events, rendering, etc.
    ///     break; // Actual loop would continue
    /// }
    /// # Ok::<(), notify::Error>(())
    /// ```
    ///
    /// # Performance
    ///
    /// This method is very fast (typically <1μs) when no events are pending.
    /// It's safe to call on every iteration of the event loop.
    pub fn check_for_changes(&mut self) -> bool {
        let mut has_changes = false;

        // Drain all pending events (non-blocking via try_recv)
        while let Ok(event_result) = self.receiver.try_recv() {
            match event_result {
                Ok(_events) => {
                    // Any successful event means the file changed
                    // We don't need to inspect the event details - any change triggers reload
                    has_changes = true;
                }
                Err(errors) => {
                    // Log errors but don't crash the application
                    // Watcher errors are not fatal - hot reload just won't work
                    eprintln!("Config watcher error: {:?}", errors);
                }
            }
        }

        has_changes
    }
}
