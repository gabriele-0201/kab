# Send and Sync
+ Send -> data is safe to be sent to another thread
+ Sync -> is safe to share between different threads the same data

Major exception include:
+ raw pointer (neither)
+ UnsafeCell isn't Sync
+ Rc isn't Send or Sync

Those are trait that can be unsafly implemented

`unsafe impl Send/Sync for Foo {}`

# UnsafeCell

Core primitive for interior mutability

# MaybeUninit
