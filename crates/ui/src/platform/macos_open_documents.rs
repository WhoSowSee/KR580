use objc2::rc::Retained;
use objc2::{ClassType, MainThreadOnly, define_class, msg_send, sel};
use objc2_core_services::{kAEOpenDocuments, kCoreEventClass, keyDirectObject};
use objc2_foundation::{NSAppleEventDescriptor, NSAppleEventManager, NSObject};
use std::cell::RefCell;
use std::path::PathBuf;

thread_local! {
    static HANDLER: RefCell<Option<Retained<OpenDocumentHandler>>> = const { RefCell::new(None) };
    static PENDING_PATH: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

define_class!(
    // SAFETY: This no-ivar main-thread class inherits NSObject's layout and lifetime rules.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct OpenDocumentHandler;

    impl OpenDocumentHandler {
        // SAFETY: This selector takes two descriptor objects and returns void, matching Apple's handler ABI.
        #[unsafe(method(handleOpenDocuments:withReplyEvent:))]
        fn handle_open_documents(
            &self,
            event: &NSAppleEventDescriptor,
            _reply: &NSAppleEventDescriptor,
        ) {
            let Some(documents) = event.paramDescriptorForKeyword(keyDirectObject) else {
                return;
            };
            for index in 1..=documents.numberOfItems() {
                let Some(path) = documents
                    .descriptorAtIndex(index)
                    .and_then(|descriptor| descriptor.fileURLValue())
                    .and_then(|url| url.path())
                    .map(|path| PathBuf::from(path.to_string()))
                else {
                    continue;
                };
                PENDING_PATH.with(|slot| {
                    if let Ok(mut slot) = slot.try_borrow_mut() {
                        *slot = Some(path);
                    }
                });
                return;
            }
        }
    }
);

pub fn install() {
    let Some(_main_thread) = objc2::MainThreadMarker::new() else {
        return;
    };
    HANDLER.with_borrow_mut(|slot| {
        if slot.is_some() {
            return;
        }
        let handler: Retained<OpenDocumentHandler> =
            // SAFETY: MainThreadMarker validated this thread; NSObject new returns a retained instance of this no-ivar subclass.
            unsafe { msg_send![OpenDocumentHandler::class(), new] };
        let manager = NSAppleEventManager::sharedAppleEventManager();
        // SAFETY: The selector matches the handler method and `slot` retains the receiver for app lifetime.
        unsafe {
            manager.setEventHandler_andSelector_forEventClass_andEventID(
                &handler,
                sel!(handleOpenDocuments:withReplyEvent:),
                kCoreEventClass,
                kAEOpenDocuments,
            );
        }
        *slot = Some(handler);
    });
}

pub fn take_pending_path() -> Option<PathBuf> {
    PENDING_PATH.with(|slot| slot.try_borrow_mut().ok()?.take())
}
