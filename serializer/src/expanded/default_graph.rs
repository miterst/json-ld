use std::hash::Hash;

use json_ld_core::{ExpandedDocument, Indexed, Object};
use linked_data::LexicalRepresentation;
use locspan::Meta;
use rdf_types::{IriVocabularyMut, Term, Vocabulary};

use crate::Error;

use super::{node::SerializeNode, value::literal_to_value};

pub struct SerializeDefaultGraph<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    result: &'a mut ExpandedDocument<V::Iri, V::BlankId>,
}

impl<'a, V: Vocabulary, I> SerializeDefaultGraph<'a, V, I> {
    pub fn new(
        vocabulary: &'a mut V,
        interpretation: &'a mut I,
        result: &'a mut ExpandedDocument<V::Iri, V::BlankId>,
    ) -> Self {
        Self {
            vocabulary,
            interpretation,
            result,
        }
    }
}

impl<'a, V: Vocabulary, I> linked_data::GraphVisitor<V, I> for SerializeDefaultGraph<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = ();
    type Error = Error;

    fn subject<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + LexicalRepresentation<V, I> + linked_data::LinkedDataSubject<V, I>,
    {
        let id = match value.lexical_representation(self.interpretation, self.vocabulary) {
            Some(Term::Literal(lit)) => {
                let value = literal_to_value(self.vocabulary, lit);
                self.result
                    .insert(Meta::none(Indexed::new(Object::Value(value), None)));
                return Ok(());
            }
            Some(Term::Id(id)) => Some(json_ld_core::Id::Valid(id)),
            None => None,
        };

        let serializer = SerializeNode::new(self.vocabulary, self.interpretation, id);

        let node = value.visit_subject(serializer)?;
        self.result
            .insert(Meta::none(Indexed::new(Object::node(node), None)));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
