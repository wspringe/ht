use anyhow::Result;
use git2::{Index, IndexAddOption, Oid, Repository};

pub trait GitCommands {
    fn add_all(&self) -> Result<Index>;
    fn tag(&self, name: &str, message: &str) -> Result<Oid>;
    fn commit(&self, index: &mut Index) -> Result<Oid>;
    fn push(&self, force: bool) -> Result<()>;
}

struct GitCli {
    repo: Repository,
}

impl GitCli {
    pub fn new(repo_path: &str) -> GitCli {
        GitCli {
            repo: Repository::open(repo_path).unwrap(),
        }
    }
}

impl GitCommands for GitCli {
    fn add_all(&self) -> Result<Index> {
        let mut index = self.repo.index().unwrap();
        index.add_all(["."], IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(index)
    }

    fn tag(&self, name: &str, message: &str) -> Result<Oid> {
        let sig = self.repo.signature()?;
        let obj = self.repo.revparse_single("HEAD")?;
        let tag = self.repo.tag(name, &obj, &sig, message, false)?;
        Ok(tag)
    }

    fn commit(&self, index: &mut Index) -> Result<Oid> {
        let signature = self.repo.signature()?;
        let tree = self.repo.find_tree(index.write_tree()?)?;
        let parent_commit = self.repo.head().unwrap().peel_to_commit()?;

        let oid = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "ci: making new version",
            &tree,
            &[&parent_commit],
        )?;
        Ok(oid)
    }

    fn push(&self, force: bool) -> Result<()> {
        let mut origin = self.repo.find_remote("origin")?;
        let head = self.repo.head()?;
        let refspec = if force {
            format!("+:refs/heads/{}", head.name().unwrap())
        } else {
            format!(
                "{}:refs/heads/{}",
                head.name().unwrap(),
                head.name().unwrap()
            )
        };
        origin.push(&[refspec], None)?;
        origin.push(&["refs/tags/*:refs/tags/*"], None)?;
        Ok(())
    }
}
