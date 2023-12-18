use recursion::{
    Collapsible, Expandable, ExpandableExt, MappableFrame, PartiallyApplied,
};
use said::{SelfAddressingIdentifier, derivation::{HashFunction, HashFunctionCode}};
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;


#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Nested {
    Said(SelfAddressingIdentifier),
    Value {
        d: SelfAddressingIdentifier,
        refs: Box<Vec<Nested>>,
    },
    // List(Box<Vec<Nested>>)
}

impl Nested {
	pub fn said(text: &str) -> Self {
		Self::Said(HashFunction::from(HashFunctionCode::Blake3_256).derive(text.as_bytes()))
	}

	pub fn value(list: Vec<Nested>) -> Self {
		let serialized_vec = serde_json::to_vec(&list).unwrap();
		Self::Value { d: HashFunction::from(HashFunctionCode::Blake3_256).derive(&serialized_vec), refs: Box::new(list) }
	}
}

pub enum NestedFrame<A> {
    Said(SelfAddressingIdentifier),
    Value {
        d: SelfAddressingIdentifier,
        refs: Vec<A>,
    },
    // List(Vec<A>)
}

impl MappableFrame for NestedFrame<PartiallyApplied> {
    type Frame<X> = NestedFrame<X>;

    fn map_frame<A, B>(input: Self::Frame<A>, mut f: impl FnMut(A) -> B) -> Self::Frame<B> {
        match input {
            NestedFrame::Said(said) => NestedFrame::Said(said),
            NestedFrame::Value { d, refs } => {
                let refs = refs.into_iter().map(|n| f(n)).collect();
                NestedFrame::Value { d, refs }
            }
            // NestedFrame::List(a) => {
            // 	NestedFrame::List(a.into_iter().map(|n| f(n)).collect())
            // },
        }
    }
}

impl Collapsible for Nested {
    type FrameToken = NestedFrame<PartiallyApplied>;

    fn into_frame(self) -> <Self::FrameToken as MappableFrame>::Frame<Self> {
        match self {
            Nested::Said(said) => NestedFrame::Said(said),
            Nested::Value { d, refs } => NestedFrame::Value { d, refs: *refs },
            // Nested::List(list) => NestedFrame::List(*list),
        }
    }
}

impl Expandable for Nested {
    type FrameToken = NestedFrame<PartiallyApplied>;

    fn from_frame(val: <Self::FrameToken as MappableFrame>::Frame<Self>) -> Self {
        match val {
            NestedFrame::Said(said) => Nested::Said(said),
            NestedFrame::Value { d, refs } => Nested::Value {
                d,
                refs: Box::new(refs),
            },
            // NestedFrame::List(list) => Nested::List(Box::new(list)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum DeserializationError {
	#[error("Missing 'd' field")]
    MissingSaid,
	#[error("Missing 'refs' field")]
    MissingRefs,
	#[error("Wrong said format")]
	WrongSaidFormat,
	#[error("Wrong refs format")]
	WrongRefsFormat,
}

impl<'de> Deserialize<'de> for Nested {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
		let input: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

		let expanded = Nested::expand_frames(input, |seed| {
			match seed {
				serde_json::Value::String(said) => NestedFrame::Said(said.parse().unwrap()),
				serde_json::Value::Object(obj) => {
					let said: Result<SelfAddressingIdentifier, _> = match obj.get("d") {
						Some(serde_json::Value::String(said_str)) => {
							said_str.parse().map_err(D::Error::custom)
						},
						None => Err(DeserializationError::MissingSaid).map_err(D::Error::custom),
						_ => Err(DeserializationError::WrongSaidFormat).map_err(D::Error::custom),
					};
					match obj.get("refs") {
						Some(serde_json::Value::Array(arr)) => {
							NestedFrame::Value {
								d: said.unwrap(),
								refs: arr.to_owned(),
							}
						}
						None => Err(DeserializationError::MissingRefs).map_err(D::Error::custom).unwrap(),
						_ => Err(DeserializationError::WrongRefsFormat).map_err(D::Error::custom).unwrap()
					}
				}
				_ => todo!(),
			}
		});
		Ok(expanded)
	}

}


#[test]
fn test() {
	let nested_example = Nested::value(vec![
		Nested::value(vec![
			Nested::value(vec![
				Nested::said("hithere"),
				Nested::value(vec![
					Nested::said("hithere1"),
					Nested::said("hithere2"),
					Nested::said("hithere3"),
					Nested::value(vec![
						Nested::value(vec![
							Nested::value(vec![
								Nested::said("hithere1"),
								Nested::said("hithere2"),
								Nested::said("hithere3"),
							])
						])
					])
				])
			]),
			Nested::said("hithere4"),
		])
	]);


    let serialized = serde_json::to_string_pretty(&nested_example).unwrap();
    println!("{}", &serialized);

	let deser: Nested = serde_json::from_str(&serialized).unwrap();

    dbg!(&deser);
	assert_eq!(serde_json::to_string(&deser).unwrap(), serde_json::to_string(&nested_example).unwrap());
}

