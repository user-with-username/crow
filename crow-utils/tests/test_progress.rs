#[cfg(test)]
mod tests {
    use crow_utils::progress::ProgressBar;

    #[test]
    fn test_progress_bar_creation() {
        let pb = ProgressBar::new(10, "Testing");
        assert_eq!(pb.pb.length(), Some(10));
        assert_eq!(pb.pb.prefix(), "Building");
        assert_eq!(pb.pb.message(), "Testing");
        pb.finish();
    }

    #[test]
    fn test_progress_bar_inc() {
        let pb = ProgressBar::new(5, "Test inc");
        assert_eq!(pb.pb.position(), 0);
        
        pb.inc();
        assert_eq!(pb.pb.position(), 1);
        
        pb.inc();
        assert_eq!(pb.pb.position(), 2);
        
        pb.finish();
    }

    #[test]
    fn test_progress_bar_inc_with_label() {
        let pb = ProgressBar::new(3, "Start");
        assert_eq!(pb.pb.message(), "Start");
        
        pb.inc_with_label("Step 1");
        assert_eq!(pb.pb.position(), 1);
        assert_eq!(pb.pb.message(), "Step 1");
        
        pb.inc_with_label("Step 2");
        assert_eq!(pb.pb.position(), 2);
        assert_eq!(pb.pb.message(), "Step 2");
        
        pb.finish();
    }

    #[test]
    fn test_progress_bar_set_label() {
        let pb = ProgressBar::new(100, "Initial");
        assert_eq!(pb.pb.message(), "Initial");
        
        pb.set_label("Updated");
        assert_eq!(pb.pb.message(), "Updated");
        
        pb.set_label("Final");
        assert_eq!(pb.pb.message(), "Final");
        
        pb.finish();
    }

    #[test]
    fn test_progress_bar_finish() {
        let pb = ProgressBar::new(1, "Finish test");
        pb.inc();
        pb.finish();
        
        assert!(pb.pb.is_finished());
    }

    #[test]
    fn test_progress_bar_status() {
        let pb = ProgressBar::new(1, "Status test");
        
        pb.status("INFO", "Test message");
        pb.status("WARN", "Warning message");
        pb.status("ERROR", "Error message");
        
        pb.finish();
    }

    #[test]
    fn test_progress_bar_multiple_instances() {
        let pb1 = ProgressBar::new(10, "Task 1");
        let pb2 = ProgressBar::new(20, "Task 2");
        
        pb1.inc();
        pb2.inc();
        
        assert_eq!(pb1.pb.position(), 1);
        assert_eq!(pb2.pb.position(), 1);
        
        pb1.finish();
        pb2.finish();
    }

    #[test]
    fn test_progress_bar_large_values() {
        let pb = ProgressBar::new(1000000, "Large task");
        assert_eq!(pb.pb.length(), Some(1000000));
        
        for _ in 0..100 {
            pb.inc();
        }
        assert_eq!(pb.pb.position(), 100);
        
        pb.finish();
    }

    #[test]
    fn test_progress_bar_zero_total() {
        let pb = ProgressBar::new(0, "Zero task");
        assert_eq!(pb.pb.length(), Some(0));
        assert_eq!(pb.pb.position(), 0);
        
        pb.inc();
        assert_eq!(pb.pb.position(), 1);
        
        pb.finish();
    }
}