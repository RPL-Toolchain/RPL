#![feature(rustc_private)]
extern crate rustc_span;

use pest_typed::ParsableTypedNode;
use rpl_meta::utils::Path;
use rpl_parser::pairs;

#[test]
fn path() {
    rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
        {
            let p1 = pairs::Path::try_parse("Vec::<u8>").unwrap();
            let p2 = pairs::Path::try_parse("std::vec::Vec").unwrap();
            let p1 = Path::from(&p1);
            let p2 = Path::from(&p2);
            assert_eq!(p1.replace_leading_ident(p2).to_string(), "std::vec::Vec::<u8>");
        }

        {
            let p1 = pairs::Path::try_parse("Vec").unwrap();
            let p2 = pairs::Path::try_parse("std::vec::Vec::<u8>").unwrap();
            let p1 = Path::from(&p1);
            let p2 = Path::from(&p2);
            assert_eq!(p1.replace_leading_ident(p2).to_string(), "std::vec::Vec::<u8>");
        }

        {
            let p1 = pairs::Path::try_parse("Vec::new").unwrap();
            let p2 = pairs::Path::try_parse("std::vec::Vec").unwrap();
            let p1 = Path::from(&p1);
            let p2 = Path::from(&p2);
            assert_eq!(p1.replace_leading_ident(p2).to_string(), "std::vec::Vec::new");
        }

        {
            let p1 = pairs::Path::try_parse("Vec::<u8>::new").unwrap();
            let p2 = pairs::Path::try_parse("std::vec::Vec").unwrap();
            let p1 = Path::from(&p1);
            let p2 = Path::from(&p2);
            assert_eq!(p1.replace_leading_ident(p2).to_string(), "std::vec::Vec::<u8>::new");
        }

        {
            let p1 = pairs::Path::try_parse("Vec::new").unwrap();
            let p2 = pairs::Path::try_parse("std::vec::Vec::<u8>").unwrap();
            let p1 = Path::from(&p1);
            let p2 = Path::from(&p2);
            assert_eq!(p1.replace_leading_ident(p2).to_string(), "std::vec::Vec::<u8>::new");
        }

        {
            let p1 = pairs::Path::try_parse("A<C>").unwrap();
            let p2 = pairs::Path::try_parse("B<D>").unwrap();
            let p1 = Path::from(&p1);
            let p2 = Path::from(&p2);
            assert_eq!(p1.replace_leading_ident(p2).to_string(), "B::<D, C>");
        }
    });
}
