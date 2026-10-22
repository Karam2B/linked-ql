# Metas

A common pattern used in this crate is to create a zero-sized "meta" for other types, like this

```rust

pub struct Todo {
    pub title: String,
    pub completed: bool,
    pub description: Option<String>,
}

pub struct todo;

impl Meta for todo {
    type Data = Todo
}

fn behavior_1<H: Meta>(meta: &H) {}
fn behavior_2<H: Meta>(meta: &H, collection: H::Data) {}
fn behavior_3<H: Meta>(meta: &H) -> H::Data { todo!() }

// but why not

fn behavior_1<C>() {}
fn behavior_2<C>(collection: C) {}
fn behavior_3<C>() -> C { todo!() }
```

The first three examples might be look redundant as compared to the second three, especially if you (for odd reasons) like turbofish syntax of `behavior_1` and `behavior_3`, but consider this use case:


```rust 

pub struct DynamicCollection {
    pub name: String,
    pub fields: Vec<(String, Box<dyn DynamicField>)>
}

impl Meta for DynamicCollection {
    type Data = JsonValue;
}

```

this open the door for a more versatile generic codebase, for example:

```rust

fn fetch_one<H>(meta: &H) -> H::Data {
    todo!()
}

fn client_side() {
    let todo: Todo = fetch_one(&todo_meta);

    assert_eq!(todo, Todo {
        title: "first_todo",
        completed: true,
        description: Some("description_1".to_string()),
    });

    // or
    let todo: JsonValue = fetch_one(&DynamicCollection { 
        name: "todo", 
        fields: vec![
            ("title", Box::new(PhantomData<String>)),
            ("done", Box::new(PhantomData<bool>)),
            ("description", Box::new(PhantomData<Option<String>>)),
        ] 
    });

    assert_eq!(todo, json!({
        "title": "first_todo",
        "done": true,
        "description": "description_1",
    }));
}

```

now you write generic code once, and you decide to either use stack-heavy fetch_one (`todo_meta`) or use heap-heavy fetch_one (`DynamicCollectionMeta`), and guess what? they are interchangeable, if you do it the right way.

A use case for this pattern in this crate is the fact that when you create an HTTP REST server, the stack-heavy fetch_one is useless, but if you wanna use an RPC, you might wanna use the stack-heavy fetch_one for performance reasons.


# Full-circle trait extentions
 
Another pattern is to use pair of traits, where one is heavy on generics, and an extention trait that is dyn-compatible, stack-heavy, and usefull for a given use case, for example:

```rust

pub trait CollectionMeta {
    type Data;
    fn fetch_one() -> Self::Data;
}

impl CollectionMeta for todo {
    type Data = Todo;
    fn fetch_one() -> Self::Data {
        todo!()
    }
}

impl CollectionMeta for category {
    type Data = Category;
    fn fetch_one() -> Self::Data {
        todo!()
    }
}

impl CollectionMeta for tags {
    type Data = Tag;
    fn fetch_one() -> Self::Data {
        todo!()
    }
}

impl CollectionMeta for DynamicCollection {
    type Data = JsonValue;
    fn fetch_one(&self) -> Self::Data {
        todo!()
    }
}

pub trait JsonCollection {
    fn fetch_one(&self) -> JsonValue;
}

// this is automatically implemented for todo, category, tags, and DynamicCollection. 
impl<T> JsonCollection for T 
where 
    T: CollectionMeta,
    T::Data: Serialize + Deserialize,
{}

// full-circle trait extentions
impl<'generic> CollectionMeta for Box<dyn JsonCollection + 'generic>
{
    type Data = JsonValue;
    fn fetch_one() -> Self::Data {
        todo!()
    }
}

// full-circle trait extentions
impl<'generic> CollectionMeta for Arc<dyn JsonCollection + 'generic>
{
    type Data = JsonValue;
    fn fetch_one() -> Self::Data {
        todo!()
    }
}
```

Now in generic context, you use `T: CollectionMeta` in where predicates, but you are free to construct `Box<dyn JsonCollection>` and pass them to any generic function that is expecting `T: CollectionMeta`.

This is allows versatile generic code, and freedom to switch between stack-heavy and heap-heavy code, just like meta pattern.

# new sqlx::Executor trait

There is a problematic code in sqlx which is:

```rust
impl<'c> Executor<'c> for &'c mut SqliteConnection {
    type Database = Sqlite;
    fn fetch_all<'e, 'q, E>(
        self,
        query: E,
    ) where 
        'c: 'e,
        'q: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        panic!("details omitted")
    }
}
```

I created a new trait `Executor2` which change this to:

```rust
impl<'c> Executor2 for &'c mut SqliteConnection {
    type Database = Sqlite;
    fn fetch_all<'e, E>(
        self,
        query: E,
    ) where 
        E: 'e + Execute<'e, Self::Database>,
    {
        panic!("details omitted")
    }
}
```

for the following reasons:
1. the syntax `'c: 'e` is often thought of as "e" is "subtype" of "c", in my experience, this is confusing jargon, that line means "e" is THE SAME AS "c", I won't go through the proof here, but the old trait equates all "c", "e", and "q", while the new trait keeps "c" and "e" distinct. This turns out to solve all the lifetime issues I had without introducing any unsafe code.

2. no introduction of unsafe code, or changing dependencies, the only thing I did is shifting from "flume::send_async" to "flume::send", because the former is just weird. It looks like this:

```rust
// inside flume
impl<T> Sender<T>
{
    fn send_async<'wtf, T>(&'wtf self, item: T) -> SendFut<'wtf, T> {
        SendFut {
            sender: self,
            item: item,
        }
    }
}
```

I have no clue what the hell is this method, in multi-producer channels, senders are cheaply clonable, why you keep a reference to the sender inside SendFut? this was the method that created lifetime issues. 

In fact, `flume::Sender` is an `Arc<..>`, so IT IS cloneable.

The "SendFut" is a future because if the channel is bounded and full, the future will yield to the async runtime, but whatever the implementation of that future, it doesn't have to have a lifetime parameter.

So one drawback of this change is that if the channel is full, this will be blocking, but no performance cost justifies bad signature, send_async has to clone the sender, and return `SendFut<'static, T>`. or something else that is static.


