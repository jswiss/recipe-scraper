use super::models::DietaryFlag;
use std::collections::HashMap;

pub struct CuisineEntry {
    pub label: &'static str,
    pub title_keywords: &'static [&'static str],
    pub ingredient_keywords: &'static [&'static str],
    pub instruction_keywords: &'static [&'static str],
}

pub struct CourseEntry {
    pub label: &'static str,
    pub title_keywords: &'static [&'static str],
    pub ingredient_keywords: &'static [&'static str],
    pub contextual_keywords: &'static [&'static str],
}

pub struct DietDefinition {
    pub label: &'static str,
    pub excluded_flags: &'static [DietaryFlag],
}

pub fn cuisine_vocabulary() -> &'static [CuisineEntry] {
    static VOCAB: &[CuisineEntry] = &[
        CuisineEntry { label: "Italian", title_keywords: &["italian", "pasta", "risotto", "lasagna", "pizza", "gnocchi", "bruschetta", "carbonara", "bolognese", "primavera", "marinara", "parmigiana", "antipasto"], ingredient_keywords: &["parmesan", "mozzarella", "ricotta", "prosciutto", "pancetta", "basil", "oregano", "olive oil", "balsamic"], instruction_keywords: &["al dente", "sauté in olive oil"] },
        CuisineEntry { label: "Mexican", title_keywords: &["mexican", "taco", "burrito", "enchilada", "quesadilla", "tamale", "pozole", "mole", "chilaquiles", "elote"], ingredient_keywords: &["tortilla", "jalapeño", "cilantro", "cumin", "chili powder", "black beans", "avocado", "lime", "queso", "salsa", "chipotle"], instruction_keywords: &["char", "roast peppers"] },
        CuisineEntry { label: "Japanese", title_keywords: &["japanese", "sushi", "ramen", "teriyaki", "tempura", "udon", "soba", "miso", "onigiri", "gyoza", "katsu", "donburi"], ingredient_keywords: &["soy sauce", "mirin", "sake", "dashi", "nori", "wasabi", "tofu", "rice vinegar", "sesame oil", "panko"], instruction_keywords: &["roll sushi", "deep-fry"] },
        CuisineEntry { label: "Chinese", title_keywords: &["chinese", "stir-fry", "kung pao", "chow mein", "fried rice", "dim sum", "wonton", "mapo tofu", "sweet and sour", "lo mein", "peking"], ingredient_keywords: &["soy sauce", "oyster sauce", "hoisin", "five spice", "sesame oil", "ginger", "star anise", "szechuan pepper", "rice wine", "cornstarch"], instruction_keywords: &["wok", "stir-fry", "steam"] },
        CuisineEntry { label: "Thai", title_keywords: &["thai", "pad thai", "pad see ew", "tom yum", "tom kha", "green curry", "red curry", "massaman", "som tum", "larb"], ingredient_keywords: &["fish sauce", "lemongrass", "galangal", "thai basil", "coconut milk", "palm sugar", "tamarind", "kaffir lime", "thai chili"], instruction_keywords: &["wok", "stir-fry", "pound in mortar"] },
        CuisineEntry { label: "Indian", title_keywords: &["indian", "curry", "tikka masala", "biryani", "dal", "naan", "tandoori", "samosa", "paneer", "vindaloo", "korma", "chana"], ingredient_keywords: &["turmeric", "cumin", "coriander", "garam masala", "cardamom", "ghee", "yogurt", "chickpeas", "basmati", "fenugreek"], instruction_keywords: &["temper spices", "tandoor"] },
        CuisineEntry { label: "French", title_keywords: &["french", "coq au vin", "ratatouille", "bouillabaisse", "quiche", "crêpe", "soufflé", "béchamel", "gratin", "cassoulet", "croissant"], ingredient_keywords: &["butter", "shallot", "thyme", "tarragon", "dijon", "crème fraîche", "wine", "herbes de provence"], instruction_keywords: &["flambé", "deglaze", "julienne"] },
        CuisineEntry { label: "Greek", title_keywords: &["greek", "gyro", "moussaka", "souvlaki", "spanakopita", "tzatziki", "baklava", "dolma"], ingredient_keywords: &["feta", "olive oil", "lemon", "oregano", "dill", "phyllo", "kalamata", "yogurt", "cucumber"], instruction_keywords: &["grill on skewers"] },
        CuisineEntry { label: "Mediterranean", title_keywords: &["mediterranean", "hummus", "falafel", "tabbouleh", "fattoush", "shakshuka", "pita"], ingredient_keywords: &["olive oil", "chickpeas", "tahini", "lemon", "za'atar", "sumac", "pomegranate", "bulgur", "couscous"], instruction_keywords: &["drizzle olive oil"] },
        CuisineEntry { label: "Korean", title_keywords: &["korean", "bibimbap", "bulgogi", "kimchi", "japchae", "tteokbokki", "samgyeopsal", "kimbap", "sundubu"], ingredient_keywords: &["gochujang", "gochugaru", "sesame oil", "kimchi", "soy sauce", "rice wine", "doenjang", "perilla"], instruction_keywords: &["ferment", "grill at table"] },
        CuisineEntry { label: "Vietnamese", title_keywords: &["vietnamese", "pho", "banh mi", "bun", "spring roll", "goi cuon", "com tam", "cao lau"], ingredient_keywords: &["fish sauce", "rice noodles", "lime", "bean sprouts", "mint", "cilantro", "sriracha", "lemongrass", "star anise"], instruction_keywords: &["simmer broth"] },
        CuisineEntry { label: "American", title_keywords: &["american", "burger", "hot dog", "mac and cheese", "bbq", "meatloaf", "clam chowder", "pot roast"], ingredient_keywords: &["ketchup", "mustard", "cheddar", "ground beef", "bacon", "ranch", "bbq sauce"], instruction_keywords: &["grill", "smoke"] },
        CuisineEntry { label: "Southern", title_keywords: &["southern", "fried chicken", "cornbread", "collard greens", "grits", "gumbo", "biscuit", "jambalaya"], ingredient_keywords: &["buttermilk", "cornmeal", "okra", "collard greens", "hot sauce", "bacon grease", "black-eyed peas"], instruction_keywords: &["deep-fry", "slow cook"] },
        CuisineEntry { label: "Cajun/Creole", title_keywords: &["cajun", "creole", "étouffée", "crawfish", "beignet", "po'boy", "gumbo", "jambalaya"], ingredient_keywords: &["andouille", "cayenne", "crawfish", "okra", "filé powder", "trinity", "hot sauce"], instruction_keywords: &["make roux", "blacken"] },
        CuisineEntry { label: "Caribbean", title_keywords: &["caribbean", "jerk", "plantain", "rice and peas", "ackee", "roti", "callaloo"], ingredient_keywords: &["scotch bonnet", "allspice", "thyme", "coconut milk", "plantain", "jerk seasoning", "lime", "mango"], instruction_keywords: &["jerk", "marinate overnight"] },
        CuisineEntry { label: "Middle Eastern", title_keywords: &["middle eastern", "shawarma", "kebab", "kibbeh", "mansaf", "fatayer", "labneh"], ingredient_keywords: &["tahini", "sumac", "za'atar", "pomegranate molasses", "lamb", "chickpeas", "pita", "rose water"], instruction_keywords: &["roast on spit"] },
        CuisineEntry { label: "Ethiopian", title_keywords: &["ethiopian", "injera", "doro wat", "kitfo", "tibs", "shiro"], ingredient_keywords: &["berbere", "niter kibbeh", "teff", "injera", "mitmita", "fenugreek"], instruction_keywords: &["simmer stew"] },
        CuisineEntry { label: "Moroccan", title_keywords: &["moroccan", "tagine", "couscous", "harira", "bastilla", "rfissa"], ingredient_keywords: &["ras el hanout", "preserved lemon", "saffron", "couscous", "harissa", "dates", "almonds", "cinnamon"], instruction_keywords: &["slow cook in tagine"] },
        CuisineEntry { label: "Turkish", title_keywords: &["turkish", "kebab", "pide", "lahmacun", "manti", "börek", "baklava", "köfte"], ingredient_keywords: &["sumac", "pomegranate", "yogurt", "lamb", "bulgur", "red pepper flakes", "mint"], instruction_keywords: &["grill on skewers"] },
        CuisineEntry { label: "Spanish", title_keywords: &["spanish", "paella", "tapas", "gazpacho", "croqueta", "tortilla española", "churro", "patatas bravas"], ingredient_keywords: &["saffron", "smoked paprika", "chorizo", "olive oil", "sherry", "manchego", "pimiento"], instruction_keywords: &["sear in paella pan"] },
        CuisineEntry { label: "Portuguese", title_keywords: &["portuguese", "bacalhau", "pastéis de nata", "caldo verde", "francesinha", "cataplana"], ingredient_keywords: &["salt cod", "olive oil", "piri piri", "chouriço", "bay leaf", "paprika", "port wine"], instruction_keywords: &["salt cure", "grill over charcoal"] },
        CuisineEntry { label: "German", title_keywords: &["german", "schnitzel", "bratwurst", "pretzel", "sauerkraut", "strudel", "spätzle", "sauerbraten"], ingredient_keywords: &["sauerkraut", "mustard", "caraway", "juniper", "pork", "potato", "beer"], instruction_keywords: &["bread and fry", "braise"] },
        CuisineEntry { label: "British", title_keywords: &["british", "fish and chips", "shepherd's pie", "bangers and mash", "scone", "crumpet", "yorkshire pudding", "toad in the hole"], ingredient_keywords: &["malt vinegar", "worcestershire", "clotted cream", "custard", "peas", "lamb"], instruction_keywords: &["roast", "batter and fry"] },
        CuisineEntry { label: "Irish", title_keywords: &["irish", "colcannon", "boxty", "irish stew", "soda bread", "coddle"], ingredient_keywords: &["potato", "cabbage", "lamb", "stout", "butter", "oats", "leek"], instruction_keywords: &["slow simmer"] },
        CuisineEntry { label: "Scandinavian", title_keywords: &["scandinavian", "swedish", "norwegian", "danish", "smörgåsbord", "gravlax", "meatball", "smørrebrød"], ingredient_keywords: &["dill", "lingonberry", "cardamom", "rye", "salmon", "cream", "herring", "juniper"], instruction_keywords: &["cure", "pickle"] },
        CuisineEntry { label: "Russian", title_keywords: &["russian", "borscht", "pelmeni", "blini", "beef stroganoff", "piroshki", "kvass"], ingredient_keywords: &["beet", "sour cream", "dill", "buckwheat", "cabbage", "rye", "potato"], instruction_keywords: &["simmer borscht"] },
        CuisineEntry { label: "Polish", title_keywords: &["polish", "pierogi", "bigos", "kielbasa", "placek", "żurek", "gołąbki"], ingredient_keywords: &["sauerkraut", "kielbasa", "sour cream", "dill", "potato", "mushroom", "caraway"], instruction_keywords: &["boil then fry"] },
        CuisineEntry { label: "Brazilian", title_keywords: &["brazilian", "feijoada", "pão de queijo", "coxinha", "açaí", "churrasco", "brigadeiro", "moqueca"], ingredient_keywords: &["black beans", "cassava", "palm oil", "coconut milk", "lime", "farofa", "guaraná"], instruction_keywords: &["grill on skewers", "slow cook beans"] },
        CuisineEntry { label: "Peruvian", title_keywords: &["peruvian", "ceviche", "lomo saltado", "aji de gallina", "causa", "anticucho", "pisco sour"], ingredient_keywords: &["aji amarillo", "lime", "cilantro", "potato", "corn", "quinoa", "rocoto"], instruction_keywords: &["cure in lime juice"] },
        CuisineEntry { label: "Filipino", title_keywords: &["filipino", "adobo", "sinigang", "lumpia", "lechon", "pancit", "kare-kare", "sisig"], ingredient_keywords: &["soy sauce", "vinegar", "calamansi", "fish sauce", "coconut milk", "banana ketchup", "tamarind"], instruction_keywords: &["braise in vinegar"] },
        CuisineEntry { label: "Indonesian", title_keywords: &["indonesian", "nasi goreng", "rendang", "satay", "gado-gado", "soto", "tempeh"], ingredient_keywords: &["kecap manis", "sambal", "coconut milk", "lemongrass", "galangal", "tamarind", "peanut", "tempeh"], instruction_keywords: &["grind spice paste"] },
        CuisineEntry { label: "Malaysian", title_keywords: &["malaysian", "laksa", "nasi lemak", "char kway teow", "roti canai", "satay"], ingredient_keywords: &["coconut milk", "lemongrass", "belacan", "pandan", "tamarind", "sambal", "palm sugar"], instruction_keywords: &["stir-fry on high heat"] },
        CuisineEntry { label: "Hawaiian", title_keywords: &["hawaiian", "poke", "loco moco", "kalua", "haupia", "musubi", "plate lunch"], ingredient_keywords: &["pineapple", "coconut", "macadamia", "soy sauce", "rice", "spam", "taro", "lilikoi"], instruction_keywords: &["smoke in pit"] },
        CuisineEntry { label: "Tex-Mex", title_keywords: &["tex-mex", "nachos", "fajita", "chili con carne", "queso dip", "chimichanga", "taco salad"], ingredient_keywords: &["cheddar", "ground beef", "refried beans", "jalapeño", "sour cream", "chili powder", "cumin", "tortilla chips"], instruction_keywords: &["smother in cheese"] },
    ];
    VOCAB
}

pub fn course_vocabulary() -> &'static [CourseEntry] {
    static VOCAB: &[CourseEntry] = &[
        CourseEntry { label: "breakfast", title_keywords: &["breakfast", "morning", "pancake", "waffle", "omelet", "omelette", "frittata", "scramble", "granola", "porridge", "cereal"], ingredient_keywords: &["eggs", "bacon", "pancake mix", "maple syrup", "oatmeal", "cereal", "breakfast sausage"], contextual_keywords: &["overnight", "morning", "sunrise", "brunch"] },
        CourseEntry { label: "brunch", title_keywords: &["brunch", "eggs benedict", "mimosa", "quiche"], ingredient_keywords: &["eggs", "hollandaise", "smoked salmon", "champagne"], contextual_keywords: &["weekend", "late morning", "mid-morning"] },
        CourseEntry { label: "lunch", title_keywords: &["lunch", "sandwich", "wrap", "panini", "sub"], ingredient_keywords: &["bread", "deli meat", "lettuce", "tomato"], contextual_keywords: &["midday", "noon", "lunchbox", "packed lunch"] },
        CourseEntry { label: "dinner", title_keywords: &["dinner", "supper", "entrée", "main dish", "roast", "steak", "casserole"], ingredient_keywords: &["roast", "steak", "whole chicken", "pork chop", "salmon fillet"], contextual_keywords: &["evening", "tonight", "weeknight", "family meal"] },
        CourseEntry { label: "appetizer", title_keywords: &["appetizer", "starter", "hors d'oeuvre", "crostini", "bruschetta", "dip", "canapé"], ingredient_keywords: &["crackers", "cream cheese", "phyllo", "puff pastry"], contextual_keywords: &["before dinner", "party", "finger food", "bite-size"] },
        CourseEntry { label: "side dish", title_keywords: &["side dish", "side", "coleslaw", "mashed potato", "roasted vegetables", "rice pilaf"], ingredient_keywords: &["potato", "rice", "vegetables", "beans"], contextual_keywords: &["accompaniment", "alongside", "on the side", "goes with"] },
        CourseEntry { label: "main course", title_keywords: &["main course", "main", "entrée", "center piece", "pot roast", "roast chicken", "beef stew"], ingredient_keywords: &["whole chicken", "beef roast", "pork loin", "fish fillet"], contextual_keywords: &["main event", "centerpiece", "hearty"] },
        CourseEntry { label: "dessert", title_keywords: &["dessert", "cake", "cookie", "pie", "brownie", "pudding", "ice cream", "tart", "mousse", "fudge", "cheesecake", "cupcake", "macaron", "crème brûlée", "tiramisu"], ingredient_keywords: &["sugar", "vanilla extract", "cocoa", "chocolate chips", "powdered sugar", "frosting", "whipped cream", "sprinkles"], contextual_keywords: &["sweet", "bake", "decorate", "frosting", "icing"] },
        CourseEntry { label: "snack", title_keywords: &["snack", "trail mix", "energy bar", "popcorn", "chips", "granola bar", "energy ball"], ingredient_keywords: &["nuts", "dried fruit", "seeds", "chocolate chips", "oats"], contextual_keywords: &["quick bite", "on the go", "between meals", "portable"] },
        CourseEntry { label: "beverage", title_keywords: &["beverage", "drink", "smoothie", "juice", "cocktail", "lemonade", "tea", "coffee", "milkshake", "punch"], ingredient_keywords: &["ice", "juice", "milk", "tea leaves", "coffee beans", "spirits"], contextual_keywords: &["sip", "pour", "blend", "shake", "stir"] },
        CourseEntry { label: "soup", title_keywords: &["soup", "stew", "chowder", "bisque", "broth", "consommé", "gazpacho", "minestrone", "pho"], ingredient_keywords: &["broth", "stock", "bouillon", "celery", "carrot", "onion"], contextual_keywords: &["simmer", "ladle", "bowl", "warm"] },
        CourseEntry { label: "salad", title_keywords: &["salad", "slaw", "vinaigrette", "caesar", "cobb", "waldorf", "greek salad", "caprese"], ingredient_keywords: &["lettuce", "arugula", "spinach", "dressing", "vinaigrette", "croutons"], contextual_keywords: &["toss", "fresh", "crisp", "chop"] },
    ];
    VOCAB
}

pub fn diet_vocabulary() -> &'static [DietDefinition] {
    static VOCAB: &[DietDefinition] = &[
        DietDefinition { label: "vegan", excluded_flags: &[DietaryFlag::ContainsMeat, DietaryFlag::ContainsPoultry, DietaryFlag::ContainsFish, DietaryFlag::ContainsDairy, DietaryFlag::ContainsEggs] },
        DietDefinition { label: "vegetarian", excluded_flags: &[DietaryFlag::ContainsMeat, DietaryFlag::ContainsPoultry, DietaryFlag::ContainsFish] },
        DietDefinition { label: "pescatarian", excluded_flags: &[DietaryFlag::ContainsMeat, DietaryFlag::ContainsPoultry] },
        DietDefinition { label: "gluten-free", excluded_flags: &[DietaryFlag::ContainsGluten] },
        DietDefinition { label: "dairy-free", excluded_flags: &[DietaryFlag::ContainsDairy] },
        DietDefinition { label: "nut-free", excluded_flags: &[DietaryFlag::ContainsNuts] },
        DietDefinition { label: "egg-free", excluded_flags: &[DietaryFlag::ContainsEggs] },
        DietDefinition { label: "soy-free", excluded_flags: &[DietaryFlag::ContainsSoy] },
        DietDefinition { label: "keto", excluded_flags: &[DietaryFlag::HighCarb, DietaryFlag::ContainsSugar] },
        DietDefinition { label: "paleo", excluded_flags: &[DietaryFlag::ContainsGluten, DietaryFlag::ContainsDairy, DietaryFlag::ContainsSoy, DietaryFlag::ContainsSugar] },
        DietDefinition { label: "whole30", excluded_flags: &[DietaryFlag::ContainsGluten, DietaryFlag::ContainsDairy, DietaryFlag::ContainsSoy, DietaryFlag::ContainsSugar] },
        DietDefinition { label: "low-carb", excluded_flags: &[DietaryFlag::HighCarb] },
        DietDefinition { label: "low-fat", excluded_flags: &[DietaryFlag::HighFat] },
        DietDefinition { label: "sugar-free", excluded_flags: &[DietaryFlag::ContainsSugar] },
        DietDefinition { label: "Mediterranean diet", excluded_flags: &[DietaryFlag::HighFat, DietaryFlag::ContainsSugar] },
    ];
    VOCAB
}

pub fn ingredient_aliases() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        // Flours
        ("ap flour", "wheat flour"), ("all-purpose flour", "wheat flour"), ("plain flour", "wheat flour"),
        ("bread flour", "wheat flour"), ("self-rising flour", "wheat flour"), ("self-raising flour", "wheat flour"),
        ("cake flour", "wheat flour"), ("pastry flour", "wheat flour"), ("whole wheat flour", "wheat flour"),
        ("whole-wheat flour", "wheat flour"), ("semolina", "wheat flour"), ("durum flour", "wheat flour"),
        // Gluten-free flours
        ("rice flour", "rice flour"), ("almond flour", "almond"), ("coconut flour", "coconut"),
        ("oat flour", "oats"), ("tapioca flour", "tapioca"), ("tapioca starch", "tapioca"),
        ("cornstarch", "corn starch"), ("corn starch", "corn starch"), ("potato starch", "potato starch"),
        // Dairy
        ("heavy cream", "cream"), ("whipping cream", "cream"), ("half and half", "cream"),
        ("double cream", "cream"), ("light cream", "cream"), ("single cream", "cream"),
        ("whole milk", "milk"), ("skim milk", "milk"), ("2% milk", "milk"),
        ("low-fat milk", "milk"), ("evaporated milk", "milk"), ("condensed milk", "milk"),
        ("sweetened condensed milk", "milk"), ("buttermilk", "milk"),
        ("unsalted butter", "butter"), ("salted butter", "butter"), ("clarified butter", "butter"),
        ("cream cheese", "cream cheese"), ("sour cream", "sour cream"),
        ("greek yogurt", "yogurt"), ("plain yogurt", "yogurt"), ("full-fat yogurt", "yogurt"),
        ("cheddar cheese", "cheese"), ("swiss cheese", "cheese"), ("gruyère", "cheese"),
        ("provolone", "cheese"), ("jack cheese", "cheese"), ("pepper jack", "cheese"),
        ("american cheese", "cheese"), ("cottage cheese", "cheese"),
        ("parmesan cheese", "parmesan"), ("parmigiano-reggiano", "parmesan"),
        ("pecorino romano", "parmesan"),
        ("mozzarella cheese", "mozzarella"), ("fresh mozzarella", "mozzarella"),
        ("ricotta cheese", "ricotta"),
        ("feta cheese", "feta"),
        ("goat cheese", "goat cheese"), ("chèvre", "goat cheese"),
        ("blue cheese", "blue cheese"), ("gorgonzola", "blue cheese"), ("roquefort", "blue cheese"),
        // Beef
        ("ground beef", "beef"), ("beef chuck", "beef"), ("stew meat", "beef"), ("sirloin", "beef"),
        ("ribeye", "beef"), ("flank steak", "beef"), ("chuck roast", "beef"), ("brisket", "beef"),
        ("beef tenderloin", "beef"), ("filet mignon", "beef"), ("new york strip", "beef"),
        ("skirt steak", "beef"), ("short ribs", "beef"), ("beef broth", "beef"),
        // Pork
        ("pork chop", "pork"), ("pork loin", "pork"), ("pork tenderloin", "pork"),
        ("pork belly", "pork"), ("pork shoulder", "pork"), ("pulled pork", "pork"),
        ("ground pork", "pork"), ("ham", "pork"), ("bacon", "pork"), ("pancetta", "pork"),
        ("prosciutto", "pork"), ("chorizo", "pork"), ("sausage", "pork"),
        ("italian sausage", "pork"), ("breakfast sausage", "pork"), ("kielbasa", "pork"),
        ("andouille", "pork"), ("bratwurst", "pork"),
        // Chicken
        ("chicken breast", "chicken"), ("chicken thigh", "chicken"), ("chicken wing", "chicken"),
        ("chicken drumstick", "chicken"), ("chicken leg", "chicken"), ("whole chicken", "chicken"),
        ("ground chicken", "chicken"), ("rotisserie chicken", "chicken"), ("chicken tender", "chicken"),
        ("chicken broth", "chicken"), ("chicken stock", "chicken"),
        // Turkey
        ("ground turkey", "turkey"), ("turkey breast", "turkey"), ("turkey leg", "turkey"),
        ("turkey bacon", "turkey"),
        // Lamb
        ("lamb chop", "lamb"), ("lamb shank", "lamb"), ("ground lamb", "lamb"),
        ("leg of lamb", "lamb"), ("rack of lamb", "lamb"), ("lamb shoulder", "lamb"),
        // Fish
        ("salmon fillet", "salmon"), ("smoked salmon", "salmon"), ("canned salmon", "salmon"),
        ("tuna steak", "tuna"), ("canned tuna", "tuna"), ("ahi tuna", "tuna"),
        ("cod fillet", "cod"), ("tilapia fillet", "tilapia"), ("halibut fillet", "halibut"),
        ("mahi mahi", "mahi mahi"), ("sea bass", "sea bass"), ("trout fillet", "trout"),
        ("catfish fillet", "catfish"), ("swordfish steak", "swordfish"), ("sardines", "sardine"),
        ("anchovies", "anchovy"), ("anchovy paste", "anchovy"),
        // Shellfish
        ("shrimp", "shrimp"), ("prawns", "shrimp"), ("jumbo shrimp", "shrimp"),
        ("crab meat", "crab"), ("lump crab", "crab"), ("lobster tail", "lobster"),
        ("scallops", "scallop"), ("bay scallops", "scallop"), ("sea scallops", "scallop"),
        ("mussels", "mussel"), ("clams", "clam"), ("oysters", "oyster"),
        ("calamari", "squid"), ("squid", "squid"), ("octopus", "octopus"),
        ("crawfish", "crawfish"), ("crayfish", "crawfish"),
        // Eggs
        ("large eggs", "egg"), ("eggs", "egg"), ("large egg", "egg"),
        ("egg white", "egg"), ("egg yolk", "egg"), ("whole egg", "egg"),
        // Nuts
        ("almonds", "almond"), ("sliced almonds", "almond"), ("almond butter", "almond"),
        ("walnuts", "walnut"), ("walnut halves", "walnut"),
        ("pecans", "pecan"), ("pecan halves", "pecan"),
        ("cashews", "cashew"), ("cashew butter", "cashew"),
        ("peanuts", "peanut"), ("peanut butter", "peanut"), ("dry roasted peanuts", "peanut"),
        ("pine nuts", "pine nut"), ("pistachios", "pistachio"),
        ("macadamia nuts", "macadamia"), ("hazelnuts", "hazelnut"), ("brazil nuts", "brazil nut"),
        // Soy
        ("soy sauce", "soy sauce"), ("tamari", "soy sauce"), ("shoyu", "soy sauce"),
        ("low-sodium soy sauce", "soy sauce"), ("dark soy sauce", "soy sauce"),
        ("light soy sauce", "soy sauce"),
        ("firm tofu", "tofu"), ("silken tofu", "tofu"), ("extra-firm tofu", "tofu"),
        ("edamame", "edamame"), ("soy milk", "soy milk"), ("tempeh", "tempeh"),
        ("miso paste", "miso"), ("white miso", "miso"), ("red miso", "miso"),
        // Sugars
        ("granulated sugar", "sugar"), ("white sugar", "sugar"), ("cane sugar", "sugar"),
        ("brown sugar", "brown sugar"), ("dark brown sugar", "brown sugar"),
        ("light brown sugar", "brown sugar"),
        ("powdered sugar", "powdered sugar"), ("confectioners sugar", "powdered sugar"),
        ("icing sugar", "powdered sugar"),
        ("honey", "honey"), ("maple syrup", "maple syrup"), ("agave nectar", "agave"),
        ("corn syrup", "corn syrup"), ("molasses", "molasses"),
        // Grains
        ("white rice", "rice"), ("brown rice", "rice"), ("jasmine rice", "rice"),
        ("basmati rice", "rice"), ("arborio rice", "rice"), ("sushi rice", "rice"),
        ("wild rice", "rice"), ("long grain rice", "rice"),
        ("dried pasta", "pasta"), ("spaghetti", "pasta"), ("penne", "pasta"),
        ("fettuccine", "pasta"), ("linguine", "pasta"), ("rigatoni", "pasta"),
        ("macaroni", "pasta"), ("orzo", "pasta"), ("egg noodles", "egg noodles"),
        ("rice noodles", "rice noodles"), ("ramen noodles", "ramen noodles"),
        ("rolled oats", "oats"), ("steel-cut oats", "oats"), ("quick oats", "oats"),
        ("instant oats", "oats"), ("old-fashioned oats", "oats"),
        ("quinoa", "quinoa"), ("couscous", "couscous"), ("bulgur wheat", "bulgur"),
        ("barley", "barley"), ("farro", "farro"),
        // Oils
        ("extra virgin olive oil", "olive oil"), ("evoo", "olive oil"),
        ("vegetable oil", "vegetable oil"), ("canola oil", "canola oil"),
        ("coconut oil", "coconut oil"), ("sesame oil", "sesame oil"),
        ("avocado oil", "avocado oil"), ("peanut oil", "peanut oil"),
        // Common
        ("baking soda", "baking soda"), ("baking powder", "baking powder"),
        ("vanilla extract", "vanilla extract"), ("pure vanilla extract", "vanilla extract"),
        ("cocoa powder", "cocoa"), ("unsweetened cocoa", "cocoa"),
        ("dark chocolate", "chocolate"), ("milk chocolate", "chocolate"),
        ("semi-sweet chocolate", "chocolate"), ("chocolate chips", "chocolate"),
        ("bittersweet chocolate", "chocolate"),
        ("coconut milk", "coconut milk"), ("coconut cream", "coconut cream"),
        ("coconut flakes", "coconut"), ("shredded coconut", "coconut"),
    ])
}

pub fn ingredient_properties() -> HashMap<&'static str, &'static [DietaryFlag]> {
    use DietaryFlag::*;
    HashMap::from([
        // Flours & grains with gluten
        ("wheat flour", &[ContainsGluten, HighCarb] as &[DietaryFlag]),
        ("pasta", &[ContainsGluten, HighCarb]),
        ("egg noodles", &[ContainsGluten, ContainsEggs, HighCarb]),
        ("ramen noodles", &[ContainsGluten, HighCarb]),
        ("barley", &[ContainsGluten, HighCarb]),
        ("couscous", &[ContainsGluten, HighCarb]),
        ("bulgur", &[ContainsGluten, HighCarb]),
        ("farro", &[ContainsGluten, HighCarb]),
        ("seitan", &[ContainsGluten]),
        // Gluten-free grains/starches
        ("rice", &[HighCarb]),
        ("rice noodles", &[HighCarb]),
        ("oats", &[HighCarb]),
        ("quinoa", &[HighCarb]),
        ("corn starch", &[HighCarb]),
        ("tapioca", &[HighCarb]),
        ("potato starch", &[HighCarb]),
        // Meat
        ("beef", &[ContainsMeat]),
        ("pork", &[ContainsMeat]),
        ("lamb", &[ContainsMeat]),
        ("veal", &[ContainsMeat]),
        ("venison", &[ContainsMeat]),
        ("bison", &[ContainsMeat]),
        ("rabbit", &[ContainsMeat]),
        ("goat", &[ContainsMeat]),
        // Poultry
        ("chicken", &[ContainsMeat, ContainsPoultry]),
        ("turkey", &[ContainsMeat, ContainsPoultry]),
        ("duck", &[ContainsMeat, ContainsPoultry]),
        ("quail", &[ContainsMeat, ContainsPoultry]),
        // Fish
        ("salmon", &[ContainsFish]),
        ("tuna", &[ContainsFish]),
        ("cod", &[ContainsFish]),
        ("tilapia", &[ContainsFish]),
        ("halibut", &[ContainsFish]),
        ("mahi mahi", &[ContainsFish]),
        ("sea bass", &[ContainsFish]),
        ("trout", &[ContainsFish]),
        ("catfish", &[ContainsFish]),
        ("swordfish", &[ContainsFish]),
        ("sardine", &[ContainsFish]),
        ("anchovy", &[ContainsFish]),
        // Shellfish (also fish for dietary purposes)
        ("shrimp", &[ContainsFish]),
        ("crab", &[ContainsFish]),
        ("lobster", &[ContainsFish]),
        ("scallop", &[ContainsFish]),
        ("mussel", &[ContainsFish]),
        ("clam", &[ContainsFish]),
        ("oyster", &[ContainsFish]),
        ("squid", &[ContainsFish]),
        ("octopus", &[ContainsFish]),
        ("crawfish", &[ContainsFish]),
        // Dairy
        ("milk", &[ContainsDairy]),
        ("cream", &[ContainsDairy, HighFat]),
        ("butter", &[ContainsDairy, HighFat]),
        ("ghee", &[ContainsDairy, HighFat]),
        ("cheese", &[ContainsDairy]),
        ("parmesan", &[ContainsDairy]),
        ("mozzarella", &[ContainsDairy]),
        ("ricotta", &[ContainsDairy]),
        ("feta", &[ContainsDairy]),
        ("goat cheese", &[ContainsDairy]),
        ("blue cheese", &[ContainsDairy]),
        ("cream cheese", &[ContainsDairy, HighFat]),
        ("sour cream", &[ContainsDairy]),
        ("yogurt", &[ContainsDairy]),
        ("whey", &[ContainsDairy]),
        // Eggs
        ("egg", &[ContainsEggs]),
        // Nuts
        ("almond", &[ContainsNuts]),
        ("walnut", &[ContainsNuts]),
        ("pecan", &[ContainsNuts]),
        ("cashew", &[ContainsNuts]),
        ("peanut", &[ContainsNuts]),
        ("pine nut", &[ContainsNuts]),
        ("pistachio", &[ContainsNuts]),
        ("macadamia", &[ContainsNuts]),
        ("hazelnut", &[ContainsNuts]),
        ("brazil nut", &[ContainsNuts]),
        // Soy
        ("soy sauce", &[ContainsSoy, ContainsGluten]),
        ("tofu", &[ContainsSoy]),
        ("tempeh", &[ContainsSoy]),
        ("edamame", &[ContainsSoy]),
        ("soy milk", &[ContainsSoy]),
        ("miso", &[ContainsSoy]),
        // Sugars
        ("sugar", &[ContainsSugar, HighCarb]),
        ("brown sugar", &[ContainsSugar, HighCarb]),
        ("powdered sugar", &[ContainsSugar, HighCarb]),
        ("honey", &[ContainsSugar, HighCarb]),
        ("maple syrup", &[ContainsSugar, HighCarb]),
        ("agave", &[ContainsSugar, HighCarb]),
        ("corn syrup", &[ContainsSugar, HighCarb]),
        ("molasses", &[ContainsSugar, HighCarb]),
        ("chocolate", &[ContainsSugar]),
        ("cocoa", &[]),
        // Fats/Oils
        ("olive oil", &[HighFat]),
        ("vegetable oil", &[HighFat]),
        ("canola oil", &[HighFat]),
        ("coconut oil", &[HighFat]),
        ("sesame oil", &[HighFat]),
        ("avocado oil", &[HighFat]),
        ("peanut oil", &[HighFat, ContainsNuts]),
        ("lard", &[ContainsMeat, HighFat]),
        // Misc
        ("baking soda", &[]),
        ("baking powder", &[]),
        ("vanilla extract", &[]),
        ("coconut milk", &[]),
        ("coconut cream", &[HighFat]),
        ("coconut", &[]),
        ("rice flour", &[HighCarb]),
        ("fish sauce", &[ContainsFish]),
        // Vegetables & fruits (no dietary flags)
        ("tomato", &[]), ("potato", &[HighCarb]), ("onion", &[]), ("garlic", &[]),
        ("carrot", &[]), ("celery", &[]), ("bell pepper", &[]), ("broccoli", &[]),
        ("spinach", &[]), ("kale", &[]), ("lettuce", &[]), ("cabbage", &[]),
        ("zucchini", &[]), ("eggplant", &[]), ("mushroom", &[]), ("corn", &[HighCarb]),
        ("peas", &[]), ("green beans", &[]), ("asparagus", &[]), ("cauliflower", &[]),
        ("sweet potato", &[HighCarb]), ("avocado", &[HighFat]), ("cucumber", &[]),
        ("lemon", &[]), ("lime", &[]), ("orange", &[]), ("apple", &[]),
        ("banana", &[HighCarb]), ("blueberries", &[]), ("strawberries", &[]),
        // Herbs & spices (no dietary flags)
        ("basil", &[]), ("cilantro", &[]), ("parsley", &[]), ("thyme", &[]),
        ("rosemary", &[]), ("oregano", &[]), ("dill", &[]), ("mint", &[]),
        ("cumin", &[]), ("paprika", &[]), ("cinnamon", &[]), ("nutmeg", &[]),
        ("ginger", &[]), ("turmeric", &[]), ("black pepper", &[]), ("cayenne", &[]),
        ("chili powder", &[]), ("garlic powder", &[]), ("onion powder", &[]),
        // Pantry staples
        ("salt", &[]), ("pepper", &[]), ("water", &[]), ("vinegar", &[]),
        ("broth", &[]), ("stock", &[]),
    ])
}
