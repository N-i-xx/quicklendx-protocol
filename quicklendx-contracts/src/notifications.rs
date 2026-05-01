#![allow(dead_code)]
use crate::protocol_limits::{
    check_string_length, MAX_NOTIFICATION_MESSAGE_LENGTH, MAX_NOTIFICATION_TITLE_LENGTH,
};
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, BytesN, Env, Map, String, Vec};

/// Notification types for different events
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationType {
    InvoiceCreated,
    InvoiceVerified,
    InvoiceStatusChanged,
    BidReceived,
    BidAccepted,
    PaymentReceived,
    PaymentOverdue,
    InvoiceDefaulted,
    SystemAlert,
    General,
}

/// Notification priority levels
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Notification delivery status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationDeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Read,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    UserNotifications(Address),
    UserPreferences(Address),
    Notification(BytesN<32>),
    NotificationType(NotificationType),
}

/// Notification statistics
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationStats {
    pub total_sent: u32,
    pub total_delivered: u32,
    pub total_read: u32,
    pub total_failed: u32,
}

/// Notification data structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct Notification {
    pub id: BytesN<32>,
    pub recipient: Address,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub title: String,
    pub message: String,
    pub related_invoice_id: Option<BytesN<32>>,
    pub created_at: u64,
    pub delivery_status: NotificationDeliveryStatus,
    pub delivered_at: Option<u64>,
    pub read_at: Option<u64>,
    pub metadata: Map<String, String>,
}

impl Notification {
    /// Create a new notification
    pub fn new(
        env: &Env,
        recipient: Address,
        notification_type: NotificationType,
        priority: NotificationPriority,
        title: String,
        message: String,
        related_invoice_id: Option<BytesN<32>>,
    ) -> Self {
        let id = env.crypto().keccak256(&Bytes::from_array(
            &env,
            &env.ledger().timestamp().to_be_bytes(),
        ));
        let created_at = env.ledger().timestamp();

        Self {
            id: id.into(),
            recipient,
            notification_type,
            priority,
            title,
            message,
            related_invoice_id,
            created_at,
            delivery_status: NotificationDeliveryStatus::Pending,
            delivered_at: None,
            read_at: None,
            metadata: Map::new(env),
        }
    }

    /// Mark notification as sent
    pub fn mark_as_sent(&mut self, timestamp: u64) {
        self.delivery_status = NotificationDeliveryStatus::Sent;
        self.delivered_at = Some(timestamp);
    }

    /// Mark notification as delivered
    pub fn mark_as_delivered(&mut self, timestamp: u64) {
        self.delivery_status = NotificationDeliveryStatus::Delivered;
        if self.delivered_at.is_none() {
            self.delivered_at = Some(timestamp);
        }
    }

    /// Mark notification as read
    pub fn mark_as_read(&mut self, timestamp: u64) {
        self.delivery_status = NotificationDeliveryStatus::Read;
        self.read_at = Some(timestamp);
    }

    /// Mark notification as failed
    pub fn mark_as_failed(&mut self) {
        self.delivery_status = NotificationDeliveryStatus::Failed;
    }
}

/// User notification preferences
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationPreferences {
    pub user: Address,
    pub invoice_created: bool,
    pub invoice_verified: bool,
    pub invoice_status_changed: bool,
    pub bid_received: bool,
    pub bid_accepted: bool,
    pub payment_received: bool,
    pub payment_overdue: bool,
    pub invoice_defaulted: bool,
    pub system_alerts: bool,
    pub general: bool,
    pub minimum_priority: NotificationPriority,
    pub updated_at: u64,
}

impl NotificationPreferences {
    /// Create default notification preferences for a user
    pub fn default_for_user(env: &Env, user: Address) -> Self {
        Self {
            user,
            invoice_created: true,
            invoice_verified: true,
            invoice_status_changed: true,
            bid_received: true,
            bid_accepted: true,
            payment_received: true,
            payment_overdue: true,
            invoice_defaulted: true,
            system_alerts: true,
            general: false,
            minimum_priority: NotificationPriority::Medium,
            updated_at: env.ledger().timestamp(),
        }
    }

    /// Check if user wants notifications for a specific type
    pub fn should_notify(
        &self,
        notification_type: &NotificationType,
        priority: &NotificationPriority,
    ) -> bool {
        // Check minimum priority first
        let priority_check = match (&self.minimum_priority, priority) {
            (NotificationPriority::Critical, NotificationPriority::Critical) => true,
            (
                NotificationPriority::High,
                NotificationPriority::High | NotificationPriority::Critical,
            ) => true,
            (
                NotificationPriority::Medium,
                NotificationPriority::Medium
                | NotificationPriority::High
                | NotificationPriority::Critical,
            ) => true,
            (NotificationPriority::Low, _) => true,
            _ => false,
        };

        if !priority_check {
            return false;
        }

        // Check notification type preferences
        match notification_type {
            NotificationType::InvoiceCreated => self.invoice_created,
            NotificationType::InvoiceVerified => self.invoice_verified,
            NotificationType::InvoiceStatusChanged => self.invoice_status_changed,
            NotificationType::BidReceived => self.bid_received,
            NotificationType::BidAccepted => self.bid_accepted,
            NotificationType::PaymentReceived => self.payment_received,
            NotificationType::PaymentOverdue => self.payment_overdue,
            NotificationType::InvoiceDefaulted => self.invoice_defaulted,
            NotificationType::SystemAlert => self.system_alerts,
            NotificationType::General => self.general,
        }
    }
}

/// Main notification system
pub struct NotificationSystem;

impl NotificationSystem {
    /// Create and store a notification
    pub fn create_notification(
        env: &Env,
        recipient: Address,
        notification_type: NotificationType,
        priority: NotificationPriority,
        title: String,
        message: String,
        related_invoice_id: Option<BytesN<32>>,
    ) -> Result<BytesN<32>, crate::errors::QuickLendXError> {
        check_string_length(&title, MAX_NOTIFICATION_TITLE_LENGTH)?;
        check_string_length(&message, MAX_NOTIFICATION_MESSAGE_LENGTH)?;

        // Check if user wants this type of notification
        let preferences = Self::get_user_preferences(env, &recipient);
        if !preferences.should_notify(&notification_type, &priority) {
            return Err(crate::errors::QuickLendXError::NotificationBlocked);
        }

        // Create notification
        let notification = Notification::new(
            env,
            recipient.clone(),
            notification_type.clone(),
            priority.clone(),
            title,
            message,
            related_invoice_id,
        );

        // Store notification
        Self::store_notification(env, &notification);

        // Add to user's notification list
        Self::add_to_user_notifications(env, &recipient, &notification.id);

        // Emit notification event
        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("notif"),),
            (
                notification.id.clone(),
                recipient,
                notification_type,
                priority,
            ),
        );

        Ok(notification.id)
    }

    /// Store a notification
    fn store_notification(env: &Env, notification: &Notification) {
        let key = Self::get_notification_key(&notification.id);
        env.storage().instance().set(&key, notification);
    }

    /// Get a notification by ID
    pub fn get_notification(env: &Env, notification_id: &BytesN<32>) -> Option<Notification> {
        let key = Self::get_notification_key(notification_id);
        env.storage().instance().get(&key)
    }

    /// Update notification status
    pub fn update_notification_status(
        env: &Env,
        notification_id: &BytesN<32>,
        status: NotificationDeliveryStatus,
    ) -> Result<(), crate::errors::QuickLendXError> {
        let mut notification = Self::get_notification(env, notification_id)
            .ok_or(crate::errors::QuickLendXError::NotificationNotFound)?;

        let timestamp = env.ledger().timestamp();

        match status {
            NotificationDeliveryStatus::Sent => notification.mark_as_sent(timestamp),
            NotificationDeliveryStatus::Delivered => notification.mark_as_delivered(timestamp),
            NotificationDeliveryStatus::Read => notification.mark_as_read(timestamp),
            NotificationDeliveryStatus::Failed => notification.mark_as_failed(),
            _ => {}
        }

        Self::store_notification(env, &notification);

        // Emit status update event
        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("n_status"),),
            (notification_id.clone(), status),
        );

        Ok(())
    }

    /// Get user notifications
    pub fn get_user_notifications(env: &Env, user: &Address) -> Vec<BytesN<32>> {
        let key = Self::get_user_notifications_key(user);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get user notification preferences
    pub fn get_user_preferences(env: &Env, user: &Address) -> NotificationPreferences {
        let key = DataKey::UserPreferences(user.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| NotificationPreferences::default_for_user(env, user.clone()))
    }

    /// Update user notification preferences
    pub fn update_user_preferences(
        env: &Env,
        user: &Address,
        preferences: NotificationPreferences,
    ) {
        let key = DataKey::UserPreferences(user.clone());
        env.storage().instance().set(&key, &preferences);

        // Emit preferences update event
        #[allow(deprecated)]
        env.events()
            .publish((symbol_short!("pref_up"),), (user.clone(),));
    }

    /// Get notification statistics for a user
    pub fn get_user_notification_stats(env: &Env, user: &Address) -> NotificationStats {
        let notifications = Self::get_user_notifications(env, user);
        let mut stats = NotificationStats {
            total_sent: 0,
            total_delivered: 0,
            total_read: 0,
            total_failed: 0,
        };

        for notification_id in notifications.iter() {
            if let Some(notification) = Self::get_notification(env, &notification_id) {
                match notification.delivery_status {
                    NotificationDeliveryStatus::Sent => stats.total_sent += 1,
                    NotificationDeliveryStatus::Delivered => {
                        stats.total_sent += 1;
                        stats.total_delivered += 1;
                    }
                    NotificationDeliveryStatus::Read => {
                        stats.total_sent += 1;
                        stats.total_delivered += 1;
                        stats.total_read += 1;
                    }
                    NotificationDeliveryStatus::Failed => stats.total_failed += 1,
                    _ => {}
                }
            }
        }

        stats
    }

    // Storage key helpers
    fn get_notification_key(notification_id: &BytesN<32>) -> DataKey {
        DataKey::Notification(notification_id.clone())
    }

    fn get_user_notifications_key(user: &Address) -> DataKey {
        DataKey::UserNotifications(user.clone())
    }

    // Helper methods for adding to lists
    fn add_to_user_notifications(env: &Env, user: &Address, notification_id: &BytesN<32>) {
        let key = Self::get_user_notifications_key(user);
        let mut notifications = Self::get_user_notifications(env, user);
        notifications.push_back(notification_id.clone());
        env.storage().instance().set(&key, &notifications);
    }
}

