use ::{TClient,TabunError,Comment,EditablePost,Post,InBlogs,UserInfo};

use std::collections::HashMap;
use select::predicate::{Class, Name, And, Attr};

impl<'a> TClient<'a> {

    ///Получить комменты из некоторого поста
    ///в виде HashMap ID-Коммент. Если блог указан как ""
    ///и пост указан как 0, то получает из `/comments/`
    ///
    ///# Examples
    ///```no_run
    ///# let mut user = libtabun::TClient::new("логин","пароль").unwrap();
    ///user.get_comments("lighthouse",157807);
    ///```
    pub fn get_comments(&mut self,blog: &str, post_id: i32) -> Result<HashMap<i64,Comment>,TabunError> {
        let mut ret = HashMap::new();

        let ref url = if blog.is_empty() && post_id == 0 {
            "/comments".to_owned()
        } else {
            format!("/blog/{}/{}.html", blog, post_id)
        };

        let page = try!(self.get(url));

        let comments = page.find(And(Name("div"),Class("comments")));

        for comm in comments.find(Class("comment")).iter() {
            let mut parent = 0i64;
            if comm.parent().unwrap().parent().unwrap().is(And(Name("div"),Class("comment-wrapper"))) {
                let p = comm.find(And(Name("li"),Class("vote"))).first().unwrap();
                parent = p.attr("id").unwrap().split("_").collect::<Vec<_>>()[3].parse::<i64>().unwrap();
            }

            let text = comm.find(And(Name("div"),Class("text"))).first().unwrap().inner_html();
            let text = text.as_str();

            let id = comm.find(And(Name("li"),Class("vote"))).first().unwrap();
            let id = id.attr("id").unwrap().split("_").collect::<Vec<_>>()[3].parse::<i64>().unwrap();

            let author = comm.find(And(Name("li"),Class("comment-author")))
                .find(Name("a"))
                .first()
                .unwrap();
            let author = author.attr("href").unwrap().split("/").collect::<Vec<_>>()[4];

            let date = comm.find(Name("time")).first().unwrap();
            let date = date.attr("datetime").unwrap();

            let votes = comm.find(And(Name("span"),Class("vote-count")))
                .first()
                .unwrap()
                .text().parse::<i32>().unwrap();
            ret.insert(id,Comment{
                body:   text.to_owned(),
                id:     id,
                author: author.to_owned(),
                date:   date.to_owned(),
                votes:  votes,
                parent: parent,
            });
        }
        return Ok(ret);
    }

    ///Получает ID блога по его имени
    ///
    ///# Examples
    ///```no_run
    ///# let mut user = libtabun::TClient::new("логин","пароль").unwrap();
    ///let blog_id = user.get_blog_id("lighthouse").unwrap();
    ///assert_eq!(blog_id,15558);
    ///```
    pub fn get_blog_id(&mut self,name: &str) -> Result<i32,TabunError> {
        use mdo::option::{bind,ret};

        let url = format!("/blog/{}", name);
        let page = try!(self.get(&url));

        Ok(mdo!(
            x =<< page.find(And(Name("div"),Class("vote-item"))).first();
            x =<< x.find(Name("span")).first();
            x =<< x.attr("id");
            x =<< x.split("_").collect::<Vec<_>>().last();
            x =<< x.parse::<i32>().ok();
            ret ret(x)
        ).unwrap())
    }

    ///Получает посты из блога
    ///
    ///# Examples
    ///```no_run
    ///# let mut user = libtabun::TClient::new("логин","пароль").unwrap();
    ///user.get_posts("lighthouse",1);
    ///```
    pub fn get_posts(&mut self, blog_name: &str, page: i32) -> Result<Vec<Post>,TabunError>{
       let res = try!(self.get(&format!("/blog/{}/page{}", blog_name, page)));
       let mut ret = Vec::new();

       for p in res.find(Name("article")).iter() {
        let post_id = p.find(And(Name("div"),Class("vote-topic")))
               .first()
               .unwrap()
               .attr("id")
               .unwrap()
               .split("_").collect::<Vec<_>>()[3].parse::<i32>().unwrap();

        let post_title = p.find(And(Name("h1"),Class("topic-title")))
            .first()
            .unwrap()
            .text();

        let post_body = p.find(And(Name("div"),Class("topic-content")))
            .first()
            .unwrap()
            .inner_html();
        let post_body = post_body.trim();

        let post_date = p.find(And(Name("li"),Class("topic-info-date")))
            .find(Name("time"))
            .first()
            .unwrap();
        let post_date = post_date.attr("datetime")
            .unwrap();

        let mut post_tags = Vec::new();
        for t in res.find(And(Name("a"),Attr("rel","tag"))).iter() {
            post_tags.push(t.text());
        }

        let cm_count = p.find(And(Name("li"),Class("topic-info-comments")))
            .first()
            .unwrap()
            .find(Name("span")).first().unwrap().text()
            .parse::<i32>().unwrap();

        let post_author = res.find(And(Name("div"),Class("topic-info")))
            .find(And(Name("a"),Attr("rel","author")))
            .first()
            .unwrap()
            .text();
        ret.push(
            Post{
                title:          post_title,
                body:           post_body.to_owned(),
                date:           post_date.to_owned(),
                tags:           post_tags,
                comments_count: cm_count,
                author:         post_author,
                id:             post_id, });
       }
       Ok(ret)
    }

    ///Получает EditablePost со страницы редактирования поста
    ///
    ///# Examples
    ///```no_run
    ///# let mut user = libtabun::TClient::new("логин","пароль").unwrap();
    ///user.get_editable_post(1111);
    ///```
    pub fn get_editable_post(&mut self, post_id: i32) -> Result<EditablePost,TabunError> {
        let res = try!(self.get(&format!("/topic/edit/{}",post_id)));

        let title = res.find(Attr("id","topic_title")).first().unwrap();
        let title = title.attr("value").unwrap().to_string();

        let tags = res.find(Attr("id","topic_tags")).first().unwrap();
        let tags = tags.attr("value").unwrap();
        let tags = tags.split(",").map(|x| x.to_string()).collect::<Vec<String>>();

        Ok(EditablePost{
            title:  title,
            body:   res.find(Attr("id","topic_text")).first().unwrap().text(),
            tags:   tags.clone()
        })
    }

    ///Получает пост, блог можно опустить (передать `""`), но лучше так не делать,
    ///дабы избежать доволнительных перенаправлений.
    ///
    ///# Examples
    ///```no_run
    ///# let mut user = libtabun::TClient::new("логин","пароль").unwrap();
    ///user.get_post("computers",157198);
    /// //или
    ///user.get_post("",157198);
    ///```
    pub fn get_post(&mut self,blog_name: &str,post_id: i32) -> Result<Post,TabunError>{
        let res = if blog_name.is_empty() {
            try!(self.get(&format!("/blog/{}.html",post_id)))
        } else {
            try!(self.get(&format!("/blog/{}/{}.html",blog_name,post_id)))
        };

        let post_title = res.find(And(Name("h1"),Class("topic-title")))
            .first()
            .unwrap()
            .text();

        let post_body = res.find(And(Name("div"),Class("topic-content")))
            .first()
            .unwrap()
            .inner_html();
        let post_body = post_body.trim();

        let post_date = res.find(And(Name("li"),Class("topic-info-date")))
            .find(Name("time"))
            .first()
            .unwrap();
        let post_date = post_date.attr("datetime")
            .unwrap();

        let mut post_tags = Vec::new();
        for t in res.find(And(Name("a"),Attr("rel","tag"))).iter() {
            post_tags.push(t.text());
        }

        let cm_count = res.find(And(Name("span"),Attr("id","count-comments")))
            .first()
            .unwrap()
            .text()
            .parse::<i32>()
            .unwrap();

        let post_author = res.find(And(Name("div"),Class("topic-info")))
            .find(And(Name("a"),Attr("rel","author")))
            .first()
            .unwrap()
            .text();

        Ok(Post{
            title:          post_title,
            body:           post_body.to_owned(),
            date:           post_date.to_owned(),
            tags:           post_tags,
            comments_count: cm_count,
            author:         post_author,
            id:             post_id,
        })
    }

    pub fn get_profile(&mut self, name: &str) -> Result<UserInfo,TabunError> {
        let name = if name.is_empty() { self.name.clone() } else { name.to_string() };
        println!("{}",name);

        let full_url = format!("/profile/{}", name);
        let page = try!(self.get(&full_url));
        let profile = page.find(And(Name("div"),Class("profile")));

        let username = profile.find(And(Name("h2"),Attr("itemprop","nickname")))
            .first()
            .unwrap()
            .text();

        let realname = match profile.find(And(Name("p"),Attr("itemprop","name")))
            .first() {
                Some(x) => x.text(),
                None => String::new()
            };

        let skill_area = profile.find(And(Name("div"),Class("strength")))
            .find(Name("div"))
            .first()
            .unwrap();
        let skill = skill_area
            .text()
            .parse::<f32>()
            .unwrap();

        let user_id = skill_area
            .attr("id")
            .unwrap()
            .split("_")
            .collect::<Vec<_>>()[2]
            .parse::<i32>()
            .unwrap();

        let rating = profile.find(Class("vote-count"))
            .find(Name("span"))
            .first()
            .unwrap()
            .text()
            .parse::<f32>().unwrap();

        let about = page.find(And(Name("div"),Class("profile-info-about")))
            .first()
            .unwrap();

        let userpic = about.find(Class("avatar"))
            .find(Name("img"))
            .first()
            .unwrap();
        let userpic = userpic
            .attr("src")
            .unwrap();

        let description = about.find(And(Name("div"),Class("text")))
            .first()
            .unwrap()
            .inner_html();

        let dotted = page.find(And(Name("ul"), Class("profile-dotted-list")));
        let dotted = dotted.iter().last().unwrap().find(Name("li"));

        let mut other_info = HashMap::<String,String>::new();

        let mut created = Vec::<String>::new();
        let mut admin = Vec::<String>::new();
        let mut moderator = Vec::<String>::new();
        let mut member= Vec::<String>::new();

        for li in dotted.iter() {
            let name = li.find(Name("span")).first().unwrap().text();
            let val = li.find(Name("strong")).first().unwrap();

            if name.contains("Создал"){
                created = val.find(Name("a")).iter().map(|x| x.text()).collect::<Vec<_>>();
            } else if name.contains("Администрирует") {
                admin = val.find(Name("a")).iter().map(|x| x.text()).collect::<Vec<_>>();
            } else if name.contains("Модерирует") {
                moderator = val.find(Name("a")).iter().map(|x| x.text()).collect::<Vec<_>>();
            } else if name.contains("Состоит") {
                member = val.find(Name("a")).iter().map(|x| x.text()).collect::<Vec<_>>();
            } else {
                other_info.insert(name.replace(":",""),val.text());
            }
        }

        let blogs = InBlogs{
            created: created,
            admin: admin,
            moderator: moderator,
            member: member
        };

        let nav = page.find(Class("nav-profile")).find(Name("li"));

        let (mut publications,mut favourites, mut friends) = (0,0,0);

        for li in nav.iter() {
            let a = li.find(Name("a")).first().unwrap().text();

            if !a.contains("Инфо") {
                 let a = a.split("(").collect::<Vec<_>>();
                 if a.len() >1 {
                     let val = a[1].to_string()
                         .replace(")","")
                         .parse::<i32>()
                         .unwrap();
                     if a[0].contains(&"Публикации") {
                         publications = val
                     } else if a[0].contains(&"Избранное") {
                         favourites = val
                     } else {
                         friends = val
                     }
                 }
            }
        }

        Ok(UserInfo{
            username:       username,
            realname:       realname,
            skill:          skill,
            id:             user_id,
            rating:         rating,
            userpic:        userpic.to_owned(),
            description:    description,
            other_info:     other_info,
            blogs:          blogs,
            publications:   publications,
            favourites:     favourites,
            friends:        friends
        })
    }
}