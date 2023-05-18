pub struct EspressoBuilder {
    cup: Cup,
    ingredients: Vec<Ingredient>,
    customer_name: String,
    size: Size
}

impl EspressoBuilder {
    fn add_ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredients.push(ingredient);
        self
    }

    fn set_name(mut self, customer_name: &String) -> Self {
        self.cup.name = customer_name;
        self
    }

    fn set_cup_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    fn add_espresso(mut self) -> Self {
        self.ingredients.push(Ingredient::Espresso);
        self
    }

    fn add_milk(mut self) -> Self {
        self.ingredients.push(Ingredient::Milk);
        self
    }

    fn build(mut self) -> Cup {
        Cup {
            size: self.size,
            contents: self.ingredients,
            client: self.customer_name
        }
    }

    fn build_latte(mut self, name: &String, size: Size) -> Cup {
        self.ingredients.push(Ingredient::Espresso);
        self.ingredients.push(Ingredient::Milk);
        Cup {
            size,
            client: name,
            contents: self.ingredients
        }
    }
}
